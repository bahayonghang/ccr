export const formatLocalDate = (date: Date) => {
  const year = date.getFullYear()
  const month = `${date.getMonth() + 1}`.padStart(2, '0')
  const day = `${date.getDate()}`.padStart(2, '0')
  return `${year}-${month}-${day}`
}

export const getLocalDateWindow = (days: number, endDate = new Date()) => {
  const normalizedDays = Math.max(1, Math.floor(days))
  const end = new Date(endDate.getFullYear(), endDate.getMonth(), endDate.getDate())
  const start = new Date(end)
  start.setDate(end.getDate() - (normalizedDays - 1))
  return { start: formatLocalDate(start), end: formatLocalDate(end) }
}
