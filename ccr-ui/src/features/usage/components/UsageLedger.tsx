import { memo, type CSSProperties, type ReactNode } from 'react'
import '../styles/usage-ledger.css'

export type UsageLedgerAlign = 'start' | 'end'
export type UsageLedgerCellKind = 'text' | 'share' | 'status'

type UsageLedgerCellBase = {
  id: string
  text: string
  align: UsageLedgerAlign
}

export type UsageLedgerTextCell = UsageLedgerCellBase & {
  kind: 'text'
  title?: string
  secondary?: string
}

export type UsageLedgerShareCell = UsageLedgerCellBase & {
  kind: 'share'
  ratio: number
}

export type UsageLedgerStatusCell = UsageLedgerCellBase & {
  kind: 'status'
  status: string
}

export type UsageLedgerCell =
  | UsageLedgerTextCell
  | UsageLedgerShareCell
  | UsageLedgerStatusCell

export type UsageLedgerColumn = {
  id: string
  header: string
  align: UsageLedgerAlign
  colTemplate: string
}

export type UsageLedgerRowData = {
  id: string
  cells: UsageLedgerCell[]
}

export type UsageLedgerProps = {
  columns: UsageLedgerColumn[]
  rows: UsageLedgerRowData[]
  maxHeight?: string
}

const STATUS_MODIFIER: Record<string, string> = {
  static: 'usage-ledger__status--static',
  snapshot: 'usage-ledger__status--snapshot',
  mixed: 'usage-ledger__status--mixed',
  legacy_alias: 'usage-ledger__status--legacy_alias',
  unpriced: 'usage-ledger__status--unpriced',
}

function cellClassName(align: UsageLedgerAlign, extra?: string): string {
  const alignClass = align === 'end' ? 'usage-ledger__cell--end' : 'usage-ledger__cell--start'
  return extra ? `usage-ledger__cell ${alignClass} ${extra}` : `usage-ledger__cell ${alignClass}`
}

function clampShare(ratio: number): number {
  if (ratio < 0) return 0
  if (ratio > 1) return 1
  return ratio
}

function renderTextCell(cell: UsageLedgerTextCell): ReactNode {
  return (
    <div className={cellClassName(cell.align)} role="cell">
      <span className="usage-ledger__primary" title={cell.title ?? cell.text}>
        {cell.text}
      </span>
      {cell.secondary ? (
        <span className="usage-ledger__secondary" title={cell.secondary}>
          {cell.secondary}
        </span>
      ) : null}
    </div>
  )
}

function renderShareCell(cell: UsageLedgerShareCell): ReactNode {
  const percent = `${clampShare(cell.ratio) * 100}%`
  return (
    <div className={cellClassName(cell.align, 'usage-ledger__cell--share')} role="cell">
      <span className="usage-ledger__share-text">{cell.text}</span>
      <span className="usage-ledger__bar">
        <span
          className="usage-ledger__bar-fill"
          style={{ '--usage-ledger-share': percent } as CSSProperties}
        />
      </span>
    </div>
  )
}

function renderStatusCell(cell: UsageLedgerStatusCell): ReactNode {
  const modifier = STATUS_MODIFIER[cell.status]
  const statusClass = modifier
    ? `usage-ledger__status ${modifier}`
    : 'usage-ledger__status'
  return (
    <div className={cellClassName(cell.align)} role="cell">
      <span className={statusClass}>{cell.text}</span>
    </div>
  )
}

function renderLedgerCell(cell: UsageLedgerCell): ReactNode {
  switch (cell.kind) {
    case 'text':
      return renderTextCell(cell)
    case 'share':
      return renderShareCell(cell)
    case 'status':
      return renderStatusCell(cell)
    default: {
      const exhaustive: never = cell
      return exhaustive
    }
  }
}

const LedgerCell = memo(function LedgerCell({ cell }: { cell: UsageLedgerCell }) {
  return renderLedgerCell(cell)
})

export const UsageLedgerRow = memo(function UsageLedgerRow({
  row,
}: {
  row: UsageLedgerRowData
}) {
  return (
    <div className="usage-ledger__row usage-ledger__row--item" role="row">
      {row.cells.map((cell) => (
        <LedgerCell key={cell.id} cell={cell} />
      ))}
    </div>
  )
})

export function UsageLedger({
  columns,
  rows,
  maxHeight = '38rem',
}: UsageLedgerProps) {
  if (rows.length === 0) return null

  const shellStyle = {
    '--usage-ledger-cols': columns.map((column) => column.colTemplate).join(' '),
    '--usage-ledger-max-height': maxHeight,
  } as CSSProperties

  return (
    <div className="usage-ledger" role="table" style={shellStyle}>
      <div className="usage-ledger__row usage-ledger__header" role="row">
        {columns.map((column) => (
          <div
            key={column.id}
            className={
              column.align === 'end'
                ? 'usage-ledger__head usage-ledger__head--end'
                : 'usage-ledger__head'
            }
            role="columnheader"
          >
            {column.header}
          </div>
        ))}
      </div>
      {rows.map((row) => (
        <UsageLedgerRow key={row.id} row={row} />
      ))}
    </div>
  )
}
