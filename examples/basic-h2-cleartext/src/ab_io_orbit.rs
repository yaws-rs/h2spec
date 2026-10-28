//! A/B Orbit I/O

pub use blueprint::{InBuffer, Left, NoRight};

/// A/B I/O Left side
#[derive(Default)]
pub struct AbIoLeft {
    pub in_bytes: Vec<u8>,
    pub in_bytes_len: usize,
    pub out_bytes: Vec<u8>,
    pub out_bytes_len: usize,
    blocked: bool,
    ready: bool,
    want_read: bool,
    want_write: bool,
    shutdown: bool,
}

impl AbIoLeft {
    /*
        pub fn copy_in(&mut self, buf: &mut [u8]) -> usize {
            self.in_bytes.extend_from_slice(buf);
            self.in_bytes_len += buf.len();
            buf.len()
    }
        */
}

impl Left for AbIoLeft {
    /// Is Left in blocked?
    fn left_in_blocked(&self) -> bool {
        self.blocked
    }
    /// Set Left in blocked
    fn set_left_in_blocked(&mut self, b: bool) {
        self.blocked = b;
    }
    /// Lengths of Input and Output of Left side
    fn left_lens(&self) -> (usize, usize) {
        (self.in_bytes_len, self.out_bytes_len)
    }
    /// Set the Lengths of Input and Output of Left side
    fn left_set_lens(&mut self, new_len_in: usize, new_len_out: usize) -> () {
        println!("left_set_lens({new_len_in}, {new_len_out})");
        let (cur_len_in, cur_len_out) = self.left_lens();

        if new_len_in < cur_len_in {
            let cut_off = cur_len_in - new_len_in;
            let cut_end = cut_off + new_len_in;

            let mut in_bytes_new = vec![0; self.in_bytes.len()];
            if new_len_in > 0 {
                in_bytes_new[0..new_len_in].copy_from_slice(&self.in_bytes[cut_off..cut_end]);
            }

            self.in_bytes = in_bytes_new;
            self.in_bytes_len = new_len_in;
        }
        if new_len_out < cur_len_out {
            let cut_off = cur_len_out - new_len_out;
            let cut_end = cut_off + new_len_in;

            let mut out_bytes_new = vec![0; self.out_bytes.len()];
            if new_len_out > 0 {
                out_bytes_new[0..new_len_out].copy_from_slice(&self.out_bytes[cut_off..cut_end]);
            }

            self.out_bytes = out_bytes_new;
            self.out_bytes_len = new_len_out;
        }
        if new_len_in > cur_len_in {
            self.in_bytes_len = new_len_in;
        }
        if new_len_out > cur_len_out {
            self.out_bytes_len = new_len_out;
        }
    }
    /// Mutable Input and Output bufs of Left side
    fn left_bufs_mut<'d>(&'d mut self) -> (InBuffer<'d>, &'d mut [u8]) {
        let b_buf_in = &mut self.in_bytes[..self.in_bytes_len];
        let b_buf_out = &mut self.out_bytes[self.out_bytes_len..];
        (InBuffer::Single(b_buf_in), b_buf_out)
    }
    /// Indicates whether the Left side is ready for Right side
    fn is_ready(&self) -> bool {
        self.ready
    }
    /// Set the layer readiness for Right side input and output
    fn set_ready(&mut self, r: bool) -> bool {
        self.ready = r;
        self.ready
    }
    /// Indicates whether the Left side wants Input
    fn left_want_read(&self) -> bool {
        self.want_read
    }
    /// Set the Left side wanting to read
    fn set_left_want_read(&mut self, r: bool) -> () {
        self.want_read = r;
    }
    /// Indicates whether the Left side wants Output
    fn left_want_write(&self) -> bool {
        self.want_write
    }
    /// Set the Left side wanting to write
    fn set_left_want_write(&mut self, w: bool) -> () {
        self.want_write = w;
    }
    /// State machine signals shutdown (e.g. peer signals close)
    fn shutdown(&mut self) -> () {
        self.shutdown = true;
    }
}

/*
/// A/B I/O Right side
#[derive(Default)]
pub struct AbIoRight {
    pub in_bytes: Vec<u8>,
    pub in_bytes_len: usize,
    out_bytes: Vec<u8>,
    out_bytes_len: usize,
    want_right_next_in: bool,
}

impl Right for AbIoRight {
    /// Output length of Right side
    fn right_lens(&self) -> (usize, usize) {
        (self.in_bytes_len, self.out_bytes_len)
    }
    /// Indicate processing of Output of Right side
    fn buf_right_out(&self) -> &[u8] {
        &self.out_bytes[..self.out_bytes_len]
    }
    /// Indicate whether Right side wants next input block
    fn wants_right_next_in(&self) -> bool {
        self.want_right_next_in
    }
    /// SM: Indicate Right side to want the next block
    fn set_wants_right_next_in(&mut self, wr: bool) -> () {
        self.want_right_next_in = wr;
    }
    /// SM: Called when all Right side Output was consumed
    fn all_sent_right_out(&mut self) -> () {
        self.out_bytes = vec![];
        self.out_bytes_len = 0;
    }
    /// Add bytes to Right output
    fn add_right_out(&mut self, new_data: &[u8]) -> () {
        self.out_bytes.extend_from_slice(new_data);
        self.out_bytes_len += new_data.len();
    }
    /// Add bytes to Right input
    fn add_right_in(&mut self, new_data: &[u8]) -> () {
        self.in_bytes.extend_from_slice(new_data);
        self.in_bytes_len += new_data.len();
    }
}
*/
