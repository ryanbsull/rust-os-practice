use crate::{println, print};
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{stream::{Stream, StreamExt}, task::AtomicWaker};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

// create "empty" type as a way to asynchronously initialize SCANCODE_QUEUE
pub struct ScancodeStream {
    // prevents the construction of the type outside of this module
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(128))
            .expect("ScancodeStream::new() should be called just once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("ERROR: SCANCODE_QUEUE uninitialized");

        if let Some(scan) = queue.pop() {
            return Poll::Ready(Some(scan));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Some(scan) => {
                WAKER.take();
                Poll::Ready(Some(scan))
            }
            None => Poll::Pending,
        }
    }
}

// Called by keyboard interrupt handler
//
// *** MUST NOT BLOCK / ALLOCATE ***
// pub(crate) makes this function only visible to lib.rs as this should
// not be able to be called directly in main.rs
pub(crate) fn add_scancode(scan: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scan) {
            println!("WARNING: keyboard queue full, dropping input");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: keyboard queue uninitialized");
    }
}

pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore);

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(keyboard_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(keyboard_event) {
                match key {
                    DecodedKey::Unicode(char) => print!("{}", char),
                    DecodedKey::RawKey(_key) => (),
                }
            }
        }
    }
}
