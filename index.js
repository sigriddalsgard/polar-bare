const binding = require('./binding')

function toExprHandle(v) {
  if (v instanceof Expr) return v._handle
  if (typeof v === 'number') {
    if (Number.isInteger(v)) return binding.exprLitI64(v)
    return binding.exprLitF64(v)
  }
  if (typeof v === 'string') return binding.exprLitStr(v)
  if (typeof v === 'boolean') return binding.exprLitBool(v)
  if (Array.isArray(v)) return binding.exprLitSeries(arrayToSeriesHandle(v))
  if (v && typeof v === 'object' && v._handle)
    return binding.exprLitSeries(v._handle)
  throw new TypeError('cannot convert value to Expr: ' + typeof v)
}

function arrayToSeriesHandle(arr) {
  if (arr.length === 0) return binding.SeriesI64('', [])
  const first = arr[0]
  if (typeof first === 'string') return binding.SeriesStr('', arr)
  if (typeof first === 'boolean') return binding.SeriesBool('', arr)
  if (typeof first === 'number') {
    return arr.every(Number.isInteger)
      ? binding.SeriesI64('', arr)
      : binding.SeriesF64('', arr)
  }
  throw new TypeError('cannot infer Series dtype from array element: ' + typeof first)
}

class Expr {
  constructor(handle) {
    this._handle = handle
  }

  alias(name) {
    return new Expr(binding.exprAlias(this._handle, name))
  }
  cast(dtype) {
    return new Expr(binding.exprCast(this._handle, dtype))
  }
  isNull() {
    return new Expr(binding.exprIsNull(this._handle))
  }
  isNotNull() {
    return new Expr(binding.exprIsNotNull(this._handle))
  }
  fillNull(other) {
    return new Expr(binding.exprFillNull(this._handle, toExprHandle(other)))
  }
  over(by) {
    return new Expr(binding.exprOver(this._handle, by))
  }
  neg() {
    return new Expr(binding.exprNeg(this._handle))
  }
  not() {
    return new Expr(binding.exprNot(this._handle))
  }

  add(other) {
    return new Expr(binding.exprAdd(this._handle, toExprHandle(other)))
  }
  sub(other) {
    return new Expr(binding.exprSub(this._handle, toExprHandle(other)))
  }
  mul(other) {
    return new Expr(binding.exprMul(this._handle, toExprHandle(other)))
  }
  div(other) {
    return new Expr(binding.exprDiv(this._handle, toExprHandle(other)))
  }
  mod(other) {
    return new Expr(binding.exprMod(this._handle, toExprHandle(other)))
  }

  eq(other) {
    return new Expr(binding.exprEq(this._handle, toExprHandle(other)))
  }
  neq(other) {
    return new Expr(binding.exprNeq(this._handle, toExprHandle(other)))
  }
  lt(other) {
    return new Expr(binding.exprLt(this._handle, toExprHandle(other)))
  }
  ltEq(other) {
    return new Expr(binding.exprLtEq(this._handle, toExprHandle(other)))
  }
  gt(other) {
    return new Expr(binding.exprGt(this._handle, toExprHandle(other)))
  }
  gtEq(other) {
    return new Expr(binding.exprGtEq(this._handle, toExprHandle(other)))
  }

  and(other) {
    return new Expr(binding.exprAnd(this._handle, toExprHandle(other)))
  }
  or(other) {
    return new Expr(binding.exprOr(this._handle, toExprHandle(other)))
  }
  xor(other) {
    return new Expr(binding.exprXor(this._handle, toExprHandle(other)))
  }

  sum() {
    return new Expr(binding.exprSum(this._handle))
  }
  mean() {
    return new Expr(binding.exprMean(this._handle))
  }
  min() {
    return new Expr(binding.exprMin(this._handle))
  }
  max() {
    return new Expr(binding.exprMax(this._handle))
  }
  median() {
    return new Expr(binding.exprMedian(this._handle))
  }
  std() {
    return new Expr(binding.exprStd(this._handle))
  }
  var() {
    return new Expr(binding.exprVar(this._handle))
  }
  count() {
    return new Expr(binding.exprCount(this._handle))
  }
  nUnique() {
    return new Expr(binding.exprNUnique(this._handle))
  }
  first() {
    return new Expr(binding.exprFirst(this._handle))
  }
  last() {
    return new Expr(binding.exprLast(this._handle))
  }
  quantile(q) {
    return new Expr(binding.exprQuantile(this._handle, q))
  }

  abs() {
    return new Expr(binding.exprAbs(this._handle))
  }
  sign() {
    return new Expr(binding.exprSign(this._handle))
  }
  sqrt() {
    return new Expr(binding.exprSqrt(this._handle))
  }
  cbrt() {
    return new Expr(binding.exprCbrt(this._handle))
  }
  pow(exponent) {
    return new Expr(binding.exprPow(this._handle, toExprHandle(exponent)))
  }
  log(base = Math.E) {
    return new Expr(binding.exprLog(this._handle, toExprHandle(base)))
  }
  log1p() {
    return new Expr(binding.exprLog1p(this._handle))
  }
  exp() {
    return new Expr(binding.exprExp(this._handle))
  }
  floor() {
    return new Expr(binding.exprFloor(this._handle))
  }
  ceil() {
    return new Expr(binding.exprCeil(this._handle))
  }
  round(decimals = 0) {
    return new Expr(binding.exprRound(this._handle, decimals))
  }
  clip(min, max) {
    return new Expr(
      binding.exprClip(this._handle, toExprHandle(min), toExprHandle(max))
    )
  }
  clipMin(min) {
    return new Expr(binding.exprClipMin(this._handle, toExprHandle(min)))
  }
  clipMax(max) {
    return new Expr(binding.exprClipMax(this._handle, toExprHandle(max)))
  }
  sin() {
    return new Expr(binding.exprSin(this._handle))
  }
  cos() {
    return new Expr(binding.exprCos(this._handle))
  }
  tan() {
    return new Expr(binding.exprTan(this._handle))
  }
  asin() {
    return new Expr(binding.exprAsin(this._handle))
  }
  acos() {
    return new Expr(binding.exprAcos(this._handle))
  }
  atan() {
    return new Expr(binding.exprAtan(this._handle))
  }
  sinh() {
    return new Expr(binding.exprSinh(this._handle))
  }
  cosh() {
    return new Expr(binding.exprCosh(this._handle))
  }
  tanh() {
    return new Expr(binding.exprTanh(this._handle))
  }

  shift(n = 1) {
    return new Expr(binding.exprShift(this._handle, toExprHandle(n)))
  }
  diff(n = 1, nullBehavior = 'ignore') {
    return new Expr(
      binding.exprDiff(this._handle, toExprHandle(n), nullBehavior)
    )
  }
  isIn(values, nullsEqual = false) {
    return new Expr(
      binding.exprIsIn(this._handle, toExprHandle(values), nullsEqual)
    )
  }
  isBetween(low, high, closed = 'both') {
    return new Expr(
      binding.exprIsBetween(
        this._handle,
        toExprHandle(low),
        toExprHandle(high),
        closed
      )
    )
  }
  rank(options = {}) {
    return new Expr(binding.exprRank(this._handle, options))
  }
  topK(k) {
    return new Expr(binding.exprTopK(this._handle, toExprHandle(k)))
  }
  bottomK(k) {
    return new Expr(binding.exprBottomK(this._handle, toExprHandle(k)))
  }

  get str() {
    return new StrExpr(this._handle)
  }
  get dt() {
    return new DtExpr(this._handle)
  }
}

class StrExpr {
  constructor(handle) {
    this._handle = handle
  }

  contains(pat, strict = true) {
    return new Expr(
      binding.exprStrContains(this._handle, toExprHandle(pat), strict)
    )
  }
  containsLiteral(pat) {
    return new Expr(
      binding.exprStrContainsLiteral(this._handle, toExprHandle(pat))
    )
  }
  startsWith(pat) {
    return new Expr(binding.exprStrStartsWith(this._handle, toExprHandle(pat)))
  }
  endsWith(pat) {
    return new Expr(binding.exprStrEndsWith(this._handle, toExprHandle(pat)))
  }
  toLowerCase() {
    return new Expr(binding.exprStrToLowercase(this._handle))
  }
  toUpperCase() {
    return new Expr(binding.exprStrToUppercase(this._handle))
  }
  lenChars() {
    return new Expr(binding.exprStrLenChars(this._handle))
  }
  lenBytes() {
    return new Expr(binding.exprStrLenBytes(this._handle))
  }
  replace(pat, value, literal = false) {
    return new Expr(
      binding.exprStrReplace(
        this._handle,
        toExprHandle(pat),
        toExprHandle(value),
        literal
      )
    )
  }
  replaceAll(pat, value, literal = false) {
    return new Expr(
      binding.exprStrReplaceAll(
        this._handle,
        toExprHandle(pat),
        toExprHandle(value),
        literal
      )
    )
  }
  slice(offset, length) {
    return new Expr(
      binding.exprStrSlice(
        this._handle,
        toExprHandle(offset),
        toExprHandle(length)
      )
    )
  }
  head(n) {
    return new Expr(binding.exprStrHead(this._handle, toExprHandle(n)))
  }
  tail(n) {
    return new Expr(binding.exprStrTail(this._handle, toExprHandle(n)))
  }
  stripChars(matches) {
    return new Expr(
      binding.exprStrStripChars(this._handle, toExprHandle(matches))
    )
  }
  stripCharsStart(matches) {
    return new Expr(
      binding.exprStrStripCharsStart(this._handle, toExprHandle(matches))
    )
  }
  stripCharsEnd(matches) {
    return new Expr(
      binding.exprStrStripCharsEnd(this._handle, toExprHandle(matches))
    )
  }
  split(by) {
    return new Expr(binding.exprStrSplit(this._handle, toExprHandle(by)))
  }
  padStart(length, fill = ' ') {
    return new Expr(
      binding.exprStrPadStart(this._handle, toExprHandle(length), fill)
    )
  }
  padEnd(length, fill = ' ') {
    return new Expr(
      binding.exprStrPadEnd(this._handle, toExprHandle(length), fill)
    )
  }
  zfill(length) {
    return new Expr(binding.exprStrZfill(this._handle, toExprHandle(length)))
  }
  find(pat, strict = true) {
    return new Expr(
      binding.exprStrFind(this._handle, toExprHandle(pat), strict)
    )
  }
  countMatches(pat, literal = false) {
    return new Expr(
      binding.exprStrCountMatches(this._handle, toExprHandle(pat), literal)
    )
  }
  reverse() {
    return new Expr(binding.exprStrReverse(this._handle))
  }
  toDate(format) {
    return new Expr(binding.exprStrToDate(this._handle, format))
  }
}

class DtExpr {
  constructor(handle) {
    this._handle = handle
  }
  year() {
    return new Expr(binding.exprDtYear(this._handle))
  }
  isoYear() {
    return new Expr(binding.exprDtIsoYear(this._handle))
  }
  quarter() {
    return new Expr(binding.exprDtQuarter(this._handle))
  }
  month() {
    return new Expr(binding.exprDtMonth(this._handle))
  }
  week() {
    return new Expr(binding.exprDtWeek(this._handle))
  }
  weekday() {
    return new Expr(binding.exprDtWeekday(this._handle))
  }
  day() {
    return new Expr(binding.exprDtDay(this._handle))
  }
  ordinalDay() {
    return new Expr(binding.exprDtOrdinalDay(this._handle))
  }
  hour() {
    return new Expr(binding.exprDtHour(this._handle))
  }
  minute() {
    return new Expr(binding.exprDtMinute(this._handle))
  }
  second() {
    return new Expr(binding.exprDtSecond(this._handle))
  }
  daysInMonth() {
    return new Expr(binding.exprDtDaysInMonth(this._handle))
  }
  monthStart() {
    return new Expr(binding.exprDtMonthStart(this._handle))
  }
  monthEnd() {
    return new Expr(binding.exprDtMonthEnd(this._handle))
  }
  strftime(format) {
    return new Expr(binding.exprDtStrftime(this._handle, format))
  }
  timestamp(timeUnit = 'us') {
    return new Expr(binding.exprDtTimestamp(this._handle, timeUnit))
  }
}

exports.Expr = Expr

function col(name) {
  if (Array.isArray(name)) return new Expr(binding.exprCols(name))
  return new Expr(binding.exprCol(name))
}

function lit(value) {
  return new Expr(toExprHandle(value))
}

function all() {
  return new Expr(binding.exprAll())
}

function when(cond) {
  return new When([{ cond: toExprHandle(cond) }])
}

class When {
  constructor(branches) {
    this._branches = branches
  }
  then(value) {
    const last = this._branches[this._branches.length - 1]
    last.then = toExprHandle(value)
    return new Then(this._branches)
  }
}

class Then {
  constructor(branches) {
    this._branches = branches
  }
  when(cond) {
    return new When([...this._branches, { cond: toExprHandle(cond) }])
  }
  otherwise(value) {
    // Right-fold: when(c1).then(v1).when(c2).then(v2).otherwise(d)
    //   ≡ when(c1).then(v1).otherwise(when(c2).then(v2).otherwise(d))
    let acc = toExprHandle(value)
    for (let i = this._branches.length - 1; i >= 0; i--) {
      const { cond, then } = this._branches[i]
      acc = binding.exprWhenThenOtherwise(cond, then, acc)
    }
    return new Expr(acc)
  }
}

exports.col = col
exports.lit = lit
exports.all = all
exports.when = when

class Series {
  constructor(handle) {
    this._handle = handle
  }

  static i8(name, values) {
    return new Series(binding.SeriesI8(name, values))
  }

  static i64(name, values) {
    return new Series(binding.SeriesI64(name, values))
  }

  static f64(name, values) {
    return new Series(binding.SeriesF64(name, values))
  }

  static bool(name, values) {
    return new Series(binding.SeriesBool(name, values))
  }

  static str(name, values) {
    return new Series(binding.SeriesStr(name, values))
  }

  clear() {
    return new Series(binding.seriesClear(this._handle))
  }

  toDataFrame() {
    return new DataFrame(binding.seriesToDataFrame(this._handle))
  }

  toFloat() {
    return new Series(binding.seriesToFloat(this._handle))
  }

  print() {
    binding.printSeries(this._handle)
  }

  sort(options = {}) {
    return new Series(binding.sortSeries(this._handle, options))
  }

  sum() {
    return binding.seriesSum(this._handle)
  }

  mean() {
    return binding.seriesMean(this._handle)
  }

  min() {
    return binding.seriesMin(this._handle)
  }

  max() {
    return binding.seriesMax(this._handle)
  }

  median() {
    return binding.seriesMedian(this._handle)
  }

  len() {
    return binding.seriesLen(this._handle)
  }

  name() {
    return binding.seriesName(this._handle)
  }

  rename(name) {
    binding.seriesRename(this._handle, name)
    return this
  }

  dtype() {
    return binding.seriesDtype(this._handle)
  }

  cast(dtype) {
    return new Series(binding.seriesCast(this._handle, dtype))
  }

  nullCount() {
    return binding.seriesNullCount(this._handle)
  }

  isNull() {
    return new Series(binding.seriesIsNull(this._handle))
  }

  isNotNull() {
    return new Series(binding.seriesIsNotNull(this._handle))
  }

  dropNulls() {
    return new Series(binding.seriesDropNulls(this._handle))
  }

  fillNull(strategy) {
    return new Series(binding.seriesFillNull(this._handle, strategy))
  }

  unique() {
    return new Series(binding.seriesUnique(this._handle))
  }

  nUnique() {
    return binding.seriesNUnique(this._handle)
  }

  head(length) {
    return new Series(binding.seriesHead(this._handle, length))
  }

  tail(length) {
    return new Series(binding.seriesTail(this._handle, length))
  }

  slice(offset, length) {
    return new Series(binding.seriesSlice(this._handle, offset, length))
  }

  reverse() {
    return new Series(binding.seriesReverse(this._handle))
  }

  shift(periods) {
    return new Series(binding.seriesShift(this._handle, periods))
  }

  argSort(options = {}) {
    return new Series(binding.seriesArgSort(this._handle, options))
  }

  argMin() {
    return binding.seriesArgMin(this._handle)
  }

  argMax() {
    return binding.seriesArgMax(this._handle)
  }

  std() {
    return binding.seriesStd(this._handle)
  }

  var() {
    return binding.seriesVar(this._handle)
  }

  quantile(q) {
    return binding.seriesQuantile(this._handle, q)
  }

  equals(other) {
    return binding.seriesEquals(this._handle, other._handle)
  }

  get(idx) {
    return binding.seriesGet(this._handle, idx)
  }

  toArray() {
    return binding.seriesToArray(this._handle)
  }

  valueCounts() {
    return new DataFrame(binding.seriesValueCounts(this._handle))
  }
}

exports.Series = Series

class GroupBy {
  constructor(handle) {
    this._handle = handle
  }

  agg(exprs) {
    const handles = exprs.map((e) => e._handle)
    return new LazyFrame(binding.groupByAgg(this._handle, handles))
  }

  sum() {
    return new LazyFrame(binding.groupBySum(this._handle))
  }

  mean() {
    return new LazyFrame(binding.groupByMean(this._handle))
  }

  min() {
    return new LazyFrame(binding.groupByMin(this._handle))
  }

  max() {
    return new LazyFrame(binding.groupByMax(this._handle))
  }

  median() {
    return new LazyFrame(binding.groupByMedian(this._handle))
  }

  count() {
    return new LazyFrame(binding.groupByCount(this._handle))
  }

  first() {
    return new LazyFrame(binding.groupByFirst(this._handle))
  }

  last() {
    return new LazyFrame(binding.groupByLast(this._handle))
  }

  std() {
    return new LazyFrame(binding.groupByStd(this._handle))
  }

  var() {
    return new LazyFrame(binding.groupByVar(this._handle))
  }

  quantile(q) {
    return new LazyFrame(binding.groupByQuantile(this._handle, q))
  }

  head(n) {
    return new LazyFrame(binding.groupByHead(this._handle, n))
  }

  tail(n) {
    return new LazyFrame(binding.groupByTail(this._handle, n))
  }
}

exports.GroupBy = GroupBy

class DataFrame {
  constructor(columns) {
    if (Array.isArray(columns)) {
      this._handle = binding.dataFrameNew(
        columns.map((column) => column._handle)
      )
    } else {
      this._handle = columns
    }
  }

  static readCSV(path) {
    return new DataFrame(binding.readCSV(path))
  }

  static readJSON(path) {
    return new DataFrame(binding.readJSON(path))
  }

  sort(columns, options = {}) {
    return new DataFrame(binding.dataFrameSort(this._handle, columns, options))
  }

  groupBy(by) {
    return new GroupBy(binding.dataFrameGroupBy(this._handle, by))
  }

  print() {
    binding.dataFramePrint(this._handle)
  }

  shape() {
    return binding.dataFrameShape(this._handle)
  }

  select(selection) {
    return new DataFrame(binding.dataFrameSelect(this._handle, selection))
  }

  head(length) {
    return new DataFrame(binding.dataFrameHead(this._handle, length))
  }

  tail(length) {
    return new DataFrame(binding.dataFrameTail(this._handle, length))
  }

  reverse() {
    return new DataFrame(binding.dataFrameReverse(this._handle))
  }

  nullCount() {
    return new DataFrame(binding.dataFrameNullCount(this._handle))
  }

  height() {
    return binding.dataFrameHeight(this._handle)
  }

  width() {
    return binding.dataFrameWidth(this._handle)
  }

  size() {
    return binding.dataFrameSize(this._handle)
  }

  isEmpty() {
    return binding.dataFrameIsEmpty(this._handle)
  }

  drop(name) {
    return new DataFrame(binding.dataFrameDrop(this._handle, name))
  }

  pop(name) {
    return new Series(binding.dataFramePop(this._handle, name))
  }

  dropNulls(subset) {
    if (subset) {
      return new DataFrame(binding.dataFrameDropNulls(this._handle, subset))
    } else {
      return new DataFrame(binding.dataFrameDropNulls(this._handle))
    }
  }

  lazy() {
    return new LazyFrame(binding.dataFrameLazy(this._handle))
  }

  column(name) {
    return new Series(binding.dataFrameColumn(this._handle, name))
  }

  columns() {
    return binding.dataFrameColumnNames(this._handle)
  }

  rename(column, name) {
    return new DataFrame(binding.dataFrameRename(this._handle, column, name))
  }

  replace(column, newCol) {
    return new DataFrame(
      binding.dataFrameReplace(this._handle, column, newCol._handle)
    )
  }

  setColumn(column, newCol) {
    return new DataFrame(
      binding.dataFrameSetColumn(this._handle, column, newCol._handle)
    )
  }

  replaceColumn(index, newCol) {
    return new DataFrame(
      binding.dataFrameReplaceColumn(this._handle, index, newCol._handle)
    )
  }

  explode(columns) {
    return new DataFrame(binding.dataFrameExplode(this._handle, columns))
  }

  apply(name, f) {
    const wrapped = (handle) => f(new Series(handle))._handle
    return new DataFrame(binding.dataFrameApply(this._handle, name, wrapped))
  }

  slice(offset, length) {
    return new DataFrame(binding.dataFrameSlice(this._handle, offset, length))
  }

  splitAt(offset) {
    return binding
      .dataFrameSplitAt(this._handle, offset)
      .map((h) => new DataFrame(h))
  }

  shift(periods) {
    return new DataFrame(binding.dataFrameShift(this._handle, periods))
  }

  concat(other) {
    return new DataFrame(binding.dataFrameVstack(this._handle, other._handle))
  }

  concatMut(other) {
    binding.dataFrameVstackMut(this._handle, other._handle)
    return this
  }

  extend(other) {
    binding.dataFrameExtend(this._handle, other._handle)
    return this
  }

  setColumnNames(names) {
    binding.dataFrameSetColumnNames(this._handle, names)
    return this
  }

  clone() {
    return new DataFrame(binding.dataFrameClone(this._handle))
  }

  isUnique() {
    return new Series(binding.dataFrameIsUnique(this._handle))
  }

  isDuplicated() {
    return new Series(binding.dataFrameIsDuplicated(this._handle))
  }

  minHorizontal() {
    return new Series(binding.dataFrameMinHorizontal(this._handle))
  }

  maxHorizontal() {
    return new Series(binding.dataFrameMaxHorizontal(this._handle))
  }

  sumHorizontal() {
    return new Series(binding.dataFrameSumHorizontal(this._handle))
  }

  meanHorizontal() {
    return new Series(binding.dataFrameMeanHorizontal(this._handle))
  }

  filter(mask) {
    return new DataFrame(binding.dataFrameFilter(this._handle, mask._handle))
  }

  withColumn(series) {
    return new DataFrame(
      binding.dataFrameWithColumn(this._handle, series._handle)
    )
  }

  withColumns(seriesArr) {
    return new DataFrame(
      binding.dataFrameWithColumns(
        this._handle,
        seriesArr.map((s) => s._handle)
      )
    )
  }

  hstack(seriesArr) {
    return new DataFrame(
      binding.dataFrameHstack(
        this._handle,
        seriesArr.map((s) => s._handle)
      )
    )
  }

  unique(subset) {
    if (subset) {
      return new DataFrame(binding.dataFrameUnique(this._handle, subset))
    }
    return new DataFrame(binding.dataFrameUnique(this._handle))
  }

  sample(n, withReplacement = false, shuffle = false, seed) {
    return new DataFrame(
      binding.dataFrameSample(this._handle, n, withReplacement, shuffle, seed)
    )
  }

  transpose() {
    return new DataFrame(binding.dataFrameTranspose(this._handle))
  }

  sum() {
    return new DataFrame(binding.dataFrameSum(this._handle))
  }

  mean() {
    return new DataFrame(binding.dataFrameMean(this._handle))
  }

  min() {
    return new DataFrame(binding.dataFrameMin(this._handle))
  }

  max() {
    return new DataFrame(binding.dataFrameMax(this._handle))
  }

  median() {
    return new DataFrame(binding.dataFrameMedian(this._handle))
  }

  equals(other) {
    return binding.dataFrameEquals(this._handle, other._handle)
  }

  estimatedSize() {
    return binding.dataFrameEstimatedSize(this._handle)
  }

  writeCSV(path) {
    binding.dataFrameWriteCSV(this._handle, path)
    return this
  }

  writeJSON(path) {
    binding.dataFrameWriteJSON(this._handle, path)
    return this
  }
}

exports.DataFrame = DataFrame

class LazyFrame {
  constructor(handle) {
    this._handle = handle
  }

  collect() {
    return new DataFrame(binding.lazyFrameCollect(this._handle))
  }

  count() {
    return new LazyFrame(binding.lazyFrameCount(this._handle))
  }

  max() {
    return new LazyFrame(binding.lazyFrameMax(this._handle))
  }

  mean() {
    return new LazyFrame(binding.lazyFrameMean(this._handle))
  }

  median() {
    return new LazyFrame(binding.lazyFrameMedian(this._handle))
  }

  min() {
    return new LazyFrame(binding.lazyFrameMin(this._handle))
  }

  sum() {
    return new LazyFrame(binding.lazyFrameSum(this._handle))
  }

  first() {
    return new LazyFrame(binding.lazyFrameFirst(this._handle))
  }

  last() {
    return new LazyFrame(binding.lazyFrameLast(this._handle))
  }

  select(columns) {
    if (columns.length > 0 && columns[0] instanceof Expr) {
      return new LazyFrame(
        binding.lazyFrameSelectExpr(
          this._handle,
          columns.map((e) => e._handle)
        )
      )
    }
    return new LazyFrame(binding.lazyFrameSelect(this._handle, columns))
  }

  filter(arg1, op, value) {
    if (arg1 instanceof Expr) {
      return new LazyFrame(
        binding.lazyFrameFilterExpr(this._handle, arg1._handle)
      )
    }
    return new LazyFrame(binding.lazyFrameFilter(this._handle, arg1, op, value))
  }

  groupBy(by) {
    return new GroupBy(binding.lazyFrameGroupBy(this._handle, by))
  }

  join(other, leftOn, rightOn, how = 'inner') {
    return new LazyFrame(
      binding.lazyFrameJoin(this._handle, other._handle, leftOn, rightOn, how)
    )
  }

  withColumn(value) {
    if (value instanceof Expr) {
      return new LazyFrame(
        binding.lazyFrameWithColumnExpr(this._handle, value._handle)
      )
    }
    return new LazyFrame(
      binding.lazyFrameWithColumn(this._handle, value._handle)
    )
  }

  withColumns(values) {
    if (values.length > 0 && values[0] instanceof Expr) {
      return new LazyFrame(
        binding.lazyFrameWithColumnsExpr(
          this._handle,
          values.map((e) => e._handle)
        )
      )
    }
    return new LazyFrame(
      binding.lazyFrameWithColumns(
        this._handle,
        values.map((s) => s._handle)
      )
    )
  }

  sort(columns, options = {}) {
    return new LazyFrame(binding.lazyFrameSort(this._handle, columns, options))
  }

  reverse() {
    return new LazyFrame(binding.lazyFrameReverse(this._handle))
  }

  head(n) {
    return new LazyFrame(binding.lazyFrameHead(this._handle, n))
  }

  tail(n) {
    return new LazyFrame(binding.lazyFrameTail(this._handle, n))
  }

  slice(offset, length) {
    return new LazyFrame(binding.lazyFrameSlice(this._handle, offset, length))
  }

  unique(subset) {
    if (subset) {
      return new LazyFrame(binding.lazyFrameUnique(this._handle, subset))
    }
    return new LazyFrame(binding.lazyFrameUnique(this._handle))
  }

  dropNulls(subset) {
    if (subset) {
      return new LazyFrame(binding.lazyFrameDropNulls(this._handle, subset))
    }
    return new LazyFrame(binding.lazyFrameDropNulls(this._handle))
  }

  rename(from, to) {
    return new LazyFrame(binding.lazyFrameRename(this._handle, from, to))
  }

  drop(columns) {
    return new LazyFrame(binding.lazyFrameDrop(this._handle, columns))
  }

  explode(columns) {
    return new LazyFrame(binding.lazyFrameExplode(this._handle, columns))
  }

  clone() {
    return new LazyFrame(binding.lazyFrameClone(this._handle))
  }

  explain(optimized = true) {
    return binding.lazyFrameExplain(this._handle, optimized)
  }

  std() {
    return new LazyFrame(binding.lazyFrameStd(this._handle))
  }

  var() {
    return new LazyFrame(binding.lazyFrameVar(this._handle))
  }

  quantile(q) {
    return new LazyFrame(binding.lazyFrameQuantile(this._handle, q))
  }
}

exports.LazyFrame = LazyFrame

function concat(frames) {
  return new DataFrame(binding.concatDataFrames(frames.map((d) => d._handle)))
}

exports.concat = concat

function version() {
  return binding.polarsVersion()
}

exports.version = version
