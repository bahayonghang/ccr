import { SIcon } from './s-icon'

interface MarketplacePaginationProps {
  currentPage: number
  totalItems: number
  pageSize: number
  onPageChange: (page: number) => void
}

const buildPages = (total: number, current: number): number[] => {
  if (total <= 7) {
    return Array.from({ length: total }, (_, index) => index + 1)
  }
  const pages: number[] = [1]
  if (current > 3) pages.push(-1)
  const start = Math.max(2, current - 1)
  const end = Math.min(total - 1, current + 1)
  for (let page = start; page <= end; page += 1) pages.push(page)
  if (current < total - 2) pages.push(-1)
  pages.push(total)
  return pages
}

export function MarketplacePagination({
  currentPage,
  totalItems,
  pageSize,
  onPageChange,
}: MarketplacePaginationProps) {
  const totalPages = Math.max(1, Math.ceil(totalItems / pageSize))
  if (totalPages <= 1) return null

  const startItem = Math.min((currentPage - 1) * pageSize + 1, totalItems)
  const endItem = Math.min(currentPage * pageSize, totalItems)
  const pages = buildPages(totalPages, currentPage)

  const goTo = (page: number) => {
    const clamped = Math.max(1, Math.min(page, totalPages))
    if (clamped !== currentPage) onPageChange(clamped)
  }

  return (
    <div className="mp-pagination">
      <span className="mp-pagination__info">
        {startItem}-{endItem} / {totalItems}
      </span>
      <div className="mp-pagination__controls">
        <button type="button" className="mp-pagination__btn" disabled={currentPage <= 1} onClick={() => goTo(1)}>
          <SIcon name="ChevronsLeft" size="w-4 h-4" />
        </button>
        <button
          type="button"
          className="mp-pagination__btn"
          disabled={currentPage <= 1}
          onClick={() => goTo(currentPage - 1)}
        >
          <SIcon name="ChevronLeft" size="w-4 h-4" />
        </button>
        {pages.map((page, position) =>
          page === -1 ? (
            <button
              key={`ellipsis-after-${pages[position - 1] ?? 'start'}`}
              type="button"
              className="mp-pagination__ellipsis"
              disabled
            >
              …
            </button>
          ) : (
            <button
              key={page}
              type="button"
              className={
                page === currentPage ? 'mp-pagination__page mp-pagination__page--active' : 'mp-pagination__page'
              }
              onClick={() => goTo(page)}
            >
              {page}
            </button>
          ),
        )}
        <button
          type="button"
          className="mp-pagination__btn"
          disabled={currentPage >= totalPages}
          onClick={() => goTo(currentPage + 1)}
        >
          <SIcon name="ChevronRight" size="w-4 h-4" />
        </button>
        <button
          type="button"
          className="mp-pagination__btn"
          disabled={currentPage >= totalPages}
          onClick={() => goTo(totalPages)}
        >
          <SIcon name="ChevronsRight" size="w-4 h-4" />
        </button>
      </div>
    </div>
  )
}
