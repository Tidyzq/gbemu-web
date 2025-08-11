macro_rules! array {
    ($def:expr; $len:expr; $([$idx:expr]=$val:expr),* $(,)?) => { {
        let mut a = [$def; $len];
        $(a[$idx] = $val;)*
        a
    } }
}

pub(super) use array;

macro_rules! bit {
    ($exp:expr, $n:literal) => {
        ($exp & (1 << $n)) != 0
    }
}

pub(super) use bit;

macro_rules! set_bit {
    ($exp:expr, $n:literal, $b:expr) => {{
        if $b {
            $exp = ($exp | (1 << $n))
        } else {
            $exp = ($exp & !(1 << $n))
        }
    }}
}

pub(super) use set_bit;

pub struct RingBuffer<T: Clone> {
    data: Vec<Option<T>>,
    head: usize,
    tail: usize,
}

impl<T: Clone> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![None; capacity],
            head: 0,
            tail: 0,
        }
    }

    pub fn len(&self) -> usize {
        (self.data.len() + self.tail - self.head) % self.data.len()
    }

    fn grow(&mut self, capacity: usize) {
        for _ in 0..capacity {
            self.data.push(None);
        }
    }

    pub fn clear(&mut self) {
        // println!("clear");
        self.head = 0;
        self.tail = 0;
    }

    pub fn push(&mut self, item: T) {
        if self.len() == self.data.len() - 1 {
            // println!("need grow {:?} {:?}", self.len(), self.data.len());
            self.grow(self.data.len());
        }
        self.data[self.tail] = Some(item);
        self.tail = (self.tail + 1) % self.data.len();
        // println!("head:{:?} tail:{:?} len:{:?}", self.head, self.tail, self.len());
    }

    pub fn pop(&mut self) -> Option<T> {
        let item = self.data[self.head].take();
        self.head = (self.head + 1) % self.data.len();
        // println!("head:{:?} tail:{:?} len:{:?}", self.head, self.tail, self.len());
        item
    }
}
