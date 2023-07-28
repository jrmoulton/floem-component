use std::{future::Future, time::Duration};

use floem::{
    crossbeam_channel,
    ext_event::create_signal_from_channel,
    reactive::{create_effect, create_rw_signal, ReadSignal, SignalGet, SignalUpdate, SignalWith},
};
use lazy_static::lazy_static;
use tokio::sync::broadcast;

pub struct Cancel {
    pub sender: broadcast::Sender<()>,
    pub receiver: broadcast::Receiver<()>,
}

lazy_static! {
    pub static ref CANCEL: Cancel = {
        let val = broadcast::channel::<()>(1);
        Cancel { sender: val.0, receiver: val.1 }
    };
}

pub async fn cancelable_task(name: String, task: impl Future) {
    let mut cancel = CANCEL.receiver.resubscribe();
    tokio::select! {
        _ = cancel.recv() => {
            println!("Cancellation signal received. Exiting task {name}..."); // Exit the loop when the cancellation signal is received
        }
        _ = task => {}
    }
}

#[derive(Clone, Copy)]
pub struct Resource<T: 'static> {
    signal: ReadSignal<Option<T>>,
    loading: ReadSignal<bool>,
}
impl<T: Clone + 'static> Resource<T> {
    pub fn loading(&self) -> bool {
        self.loading.get()
    }

    pub fn read(&self) -> Option<T> {
        self.signal.get()
    }
}

pub fn create_resource<S, T, Fu>(
    task_name: &str, source: impl Fn() -> S + 'static,
    fetcher: impl Fn(S) -> Fu + Send + Sync + 'static,
) -> Resource<T>
where
    S: Clone + std::fmt::Debug + Send + 'static,
    T: Send + 'static,
    Fu: Future<Output = T> + Send + 'static,
{
    let (tx, rx) = crossbeam_channel::unbounded();
    let (tx2, mut rx2) = tokio::sync::mpsc::unbounded_channel();
    let loading = create_rw_signal(false);
    create_effect(move |val| {
        // tracking value
        let value = source();
        // send value through channel
        if val.is_some() {
            loading.update(|val| *val = true);
            tx2.send(value).unwrap();
        }
        Some(())
    });
    tokio::task::Builder::new()
        .name(task_name)
        .spawn(cancelable_task(task_name.to_owned(), async move {
            while let Some(value) = rx2.recv().await {
                let fetched = fetcher(value).await;
                tx.send(fetched).unwrap();
            }
        }))
        .unwrap();
    let signal = create_signal_from_channel(rx);
    create_effect(move |_| {
        signal.track();
        loading.update(|val| *val = false);
    });
    Resource {
        signal,
        loading: loading.read_only(),
    }
}

pub fn create_polled_resource<T, Fu>(
    task_name: &str, interval: Duration, fetcher: impl Fn() -> Fu + Send + Sync + Clone + 'static,
) -> ReadSignal<Option<T>>
where
    T: Send + 'static,
    Fu: Future<Output = T> + Send + 'static,
{
    let (tx, rx) = crossbeam_channel::unbounded();
    tokio::task::Builder::new()
        .name(task_name)
        .spawn(cancelable_task(task_name.to_owned(), async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                interval.tick().await;
                let fetched = fetcher().await;
                tx.send(fetched).unwrap();
            }
        }))
        .unwrap();
    create_signal_from_channel(rx)
}

pub fn run_task<S, Fu>(
    task_name: &str, source: impl Fn() -> S + 'static,
    runner: impl Fn(S) -> Fu + Send + Sync + 'static,
) where
    S: Clone + std::fmt::Debug + Send + 'static,
    Fu: Future<Output = ()> + Send + 'static,
{
    let (tx2, mut rx2) = tokio::sync::mpsc::unbounded_channel();
    create_effect(move |val| {
        // tracking value
        let value = source();
        // send value through channel
        if val.is_some() {
            tx2.send(value).unwrap();
        }
        Some(())
    });
    tokio::task::Builder::new()
        .name(task_name)
        .spawn(cancelable_task(task_name.to_owned(), async move {
            while let Some(value) = rx2.recv().await {
                runner(value).await;
            }
        }))
        .unwrap();
}

pub fn run_task_if<S, Fu>(
    task_name: &str, condition: impl Fn() -> bool + 'static, source: impl Fn() -> S + 'static,
    runner: impl Fn(S) -> Fu + Send + Sync + 'static,
) where
    S: Clone + std::fmt::Debug + Send + 'static,
    Fu: Future<Output = ()> + Send + 'static,
{
    let (tx2, mut rx2) = tokio::sync::mpsc::unbounded_channel();
    let mut cancel_clone = CANCEL.receiver.resubscribe();
    create_effect(move |val| {
        // tracking value
        let value = source();
        let condition = condition();
        // send value through channel
        if condition && val.is_some() {
            tx2.send(value).unwrap();
        }
        Some(())
    });
    tokio::task::Builder::new()
        .name(task_name)
        .spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_clone.recv() => {
                        println!("Cancellation signal received. Exiting task run task if...");
                        break; // Exit the loop when the cancellation signal is received
                    }
                    _ = async {
                        while let Some(value) = rx2.recv().await {
                            runner(value).await;
                        }
                    } =>  {
                        break;
                    }
                }
            }
        })
        .unwrap();
}
