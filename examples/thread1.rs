use anyhow::{Result, anyhow};
use std::sync::mpsc::Sender;
use std::time::Duration;
use std::{sync::mpsc, thread};

#[derive(Debug)]
#[allow(dead_code)]
struct Message {
    index: usize,
    value: usize,
}

impl Message {
    fn new(index: usize, value: usize) -> Self {
        Self { index, value }
    }
}

const NUM_PRODUCERS: usize = 4;
fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel();

    // 创建producers
    for i in 0..NUM_PRODUCERS {
        let tx = tx.clone();
        thread::spawn(move || {
            let _ = producer(i, tx);
        });
    }
    drop(tx); // 释放 tx，否则 rx 无法结束

    // 创建 consumer
    let consumer = thread::spawn(move || {
        for msg in rx {
            println!("consumer: {:?}", msg);
        }
        println!("consumer exit");
        42 // 在结束时可以返回一个数据，可以是任何data struct
    });
    let secret = consumer
        .join()
        .map_err(|e| anyhow!("Thread join error: {:?}", e))?;

    println!("secret: {secret}");

    Ok(())
}

fn producer(index: usize, tx: Sender<Message>) -> Result<()> {
    loop {
        let value = rand::random::<i32>();
        tx.send(Message::new(index, value as usize))?;
        let sleep_time = rand::random::<u8>() as u64 * 10;
        thread::sleep(Duration::from_millis(sleep_time));
        if rand::random::<u8>() % 5 == 0 {
            println!("producer {} exit", index);
            break;
        }
    }
    Ok(())
}
