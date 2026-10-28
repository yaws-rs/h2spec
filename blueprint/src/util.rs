//! (crate) Utilities

pub(crate) enum FillAttempt {
    Overflow,
    Complete(usize),
}

#[inline]
pub(crate) fn try_fill_left_with(left_out_b: &mut [u8], try_adds: &[&[u8]]) -> FillAttempt {
    let mut combined_len: usize = 0;

    for try_add in try_adds {
        let try_add_len = try_add.len();
        let (adds, overflows) = combined_len.overflowing_add(try_add_len);
        if overflows {
            return FillAttempt::Overflow;
        }
        combined_len = adds;
    }

    if combined_len > left_out_b.len() {
        return FillAttempt::Overflow;
    }

    let mut offset = 0;

    for try_add in try_adds {
        let end_pos = offset + try_add.len();
        left_out_b[offset..end_pos].copy_from_slice(try_add);
        offset += try_add.len();
    }
    FillAttempt::Complete(offset)
}

#[inline]
pub(crate) fn borrow_remaining_left_b(
    left_out_b: &mut [u8],
    out_pos: usize,
    offset: usize,
) -> &mut [u8] {
    let (try_pos_change, chk_overflow) = out_pos.overflowing_add(offset);
    if chk_overflow || try_pos_change > left_out_b.len() {
        todo!(
            "overflow {chk_overflow} out_pos {out_pos} offset {offset} try_pos_change {try_pos_change} left_out_b_len {}",
            left_out_b.len()
        )
    }
    let end_pos = left_out_b.len() - out_pos - offset;
    &mut left_out_b[out_pos..end_pos]
}
