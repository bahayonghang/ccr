pub(crate) fn previous_index_in_page(page_len: usize, selected_index: usize) -> usize {
    let selected_index = clamped_index(page_len, selected_index);
    if page_len == 0 {
        0
    } else if selected_index == 0 {
        page_len - 1
    } else {
        selected_index - 1
    }
}

pub(crate) fn next_index_in_page(page_len: usize, selected_index: usize) -> usize {
    let selected_index = clamped_index(page_len, selected_index);
    if page_len <= 1 || selected_index >= page_len - 1 {
        0
    } else {
        selected_index + 1
    }
}

fn clamped_index(page_len: usize, selected_index: usize) -> usize {
    if page_len == 0 {
        0
    } else {
        selected_index.min(page_len - 1)
    }
}

#[cfg(test)]
mod navigation_wrap {
    use super::*;

    #[test]
    fn empty_page_stays_at_zero() {
        assert_eq!(previous_index_in_page(0, 0), 0);
        assert_eq!(next_index_in_page(0, 0), 0);
    }

    #[test]
    fn single_item_page_stays_at_zero() {
        assert_eq!(previous_index_in_page(1, 0), 0);
        assert_eq!(next_index_in_page(1, 0), 0);
    }

    #[test]
    fn previous_wraps_first_item_to_last_item() {
        assert_eq!(previous_index_in_page(10, 0), 9);
    }

    #[test]
    fn next_wraps_last_item_to_first_item() {
        assert_eq!(next_index_in_page(10, 9), 0);
    }

    #[test]
    fn middle_items_move_one_step() {
        assert_eq!(previous_index_in_page(10, 5), 4);
        assert_eq!(next_index_in_page(10, 5), 6);
    }

    #[test]
    fn out_of_range_index_is_clamped_before_moving() {
        assert_eq!(previous_index_in_page(3, 99), 1);
        assert_eq!(next_index_in_page(3, 99), 0);
    }
}
