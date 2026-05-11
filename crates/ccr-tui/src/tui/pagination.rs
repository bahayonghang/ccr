pub(crate) const DEFAULT_PAGE_SIZE: usize = 10;

pub(crate) fn visible_page_size(content_height: u16) -> usize {
    usize::from(content_height).max(1)
}

pub(crate) fn page_for_index(index: usize, page_size: usize) -> usize {
    index / normalized_page_size(page_size)
}

pub(crate) fn index_in_page(index: usize, page_size: usize) -> usize {
    index % normalized_page_size(page_size)
}

pub(crate) fn total_pages(item_count: usize, page_size: usize) -> usize {
    if item_count == 0 {
        1
    } else {
        item_count.div_ceil(normalized_page_size(page_size))
    }
}

pub(crate) fn page_slice<T>(items: &[T], current_page: usize, page_size: usize) -> &[T] {
    let page_size = normalized_page_size(page_size);
    let start = current_page.saturating_mul(page_size);
    let end = start.saturating_add(page_size).min(items.len());
    if start >= items.len() {
        &[]
    } else {
        &items[start..end]
    }
}

fn normalized_page_size(page_size: usize) -> usize {
    page_size.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_page_size_uses_content_height_with_minimum_one() {
        assert_eq!(visible_page_size(0), 1);
        assert_eq!(visible_page_size(1), 1);
        assert_eq!(visible_page_size(2), 2);
        assert_eq!(visible_page_size(10), 10);
        assert_eq!(visible_page_size(14), 14);
        assert_eq!(visible_page_size(200), 200);
    }

    #[test]
    fn total_pages_normalizes_zero_page_size() {
        assert_eq!(total_pages(0, 0), 1);
        assert_eq!(total_pages(1, 0), 1);
        assert_eq!(total_pages(11, 10), 2);
        assert_eq!(total_pages(20, 20), 1);
    }

    #[test]
    fn page_slice_respects_dynamic_page_size() {
        let items = [1, 2, 3, 4, 5];

        assert_eq!(page_slice(&items, 0, 3), &[1, 2, 3]);
        assert_eq!(page_slice(&items, 1, 3), &[4, 5]);
        assert_eq!(page_slice(&items, 2, 3), &[] as &[i32]);
    }
}
