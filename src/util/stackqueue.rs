const MAX_QUEUE_LEN: usize = 64;

pub struct StackQueue<T>
where
    T: Default + Copy,
{
    data: [T; MAX_QUEUE_LEN],
    head: usize,
    tail: usize,
    count: usize,
}

impl<T> StackQueue<T>
where
    T: Default + Copy,
{
    pub fn new() -> Self {
        Self {
            data: [T::default(); MAX_QUEUE_LEN],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.count == 0 {
            None
        } else {
            let idx = self.head;
            self.head += 1;
            if self.head == self.data.len() {
                self.head = 0;
            }
            self.count -= 1;

            let res = unsafe { self.data.get_unchecked(idx) };
            Some(*res)
        }
    }

    pub fn push_back(&mut self, v: T) {
        if self.count == self.data.len() {
            panic!("overfilled StackQueue")
        } else {
            let slot = unsafe { self.data.get_unchecked_mut(self.tail) };
            *slot = v;
            self.tail += 1;
            if self.tail == self.data.len() {
                self.tail = 0;
            }
            self.count += 1;
        }
    }
}
