# polar-bare

Polars bindings for Bare. Wraps the Rust Polars DataFrame library (<https://github.com/pola-rs/polars>) as a native addon, exposing eager `DataFrame`/`Series` types, a lazy query engine via `LazyFrame`, and a chainable expression DSL (`Expr`) with string and datetime accessors.

```
npm i polar-bare
```

## Usage

```js
const { DataFrame, Series, col, lit, when } = require('polar-bare')

// Build a frame from typed Series.
const df = new DataFrame([
  Series.str('name', ['Aurora', 'Felix', 'Mateo', 'Noor']),
  Series.str('dept', ['eng', 'eng', 'design', 'data']),
  Series.i64('salary', [85000, 92000, 78000, 65000])
])

df.print()

// Lazy query: tag a tier, group, aggregate.
df.lazy()
  .withColumn(
    when(col('salary').gtEq(90000))
      .then(lit('senior'))
      .otherwise(lit('other'))
      .alias('tier')
  )
  .groupBy(['dept'])
  .agg([col('salary').mean().alias('mean_salary')])
  .collect()
  .sort(['dept'])
  .print()
```

## API

### Top-level functions

#### `const expr = col(name)`

Returns an `Expr` referencing one or more columns. `name` is a column name string, or an array of names to select multiple columns at once.

#### `const expr = lit(value)`

Returns an `Expr` wrapping a literal `value`. Accepts numbers (integers become `i64`, non-integers `f64`), strings, booleans, arrays (converted to a `Series` literal), and existing `Series` instances.

#### `const expr = all()`

Returns an `Expr` selecting all columns.

#### `const branch = when(cond)`

Begins a conditional expression. `cond` is an `Expr` (or value coerced to one). Returns a `When` whose `.then(value)` sets the result for that branch and yields a `Then`. Chain further `.when().then()` pairs and close with `.otherwise(value)` to produce the final `Expr`.

```js
when(a).then(x).when(b).then(y).otherwise(z)
```

#### `const df = concat(frames)`

Vertically concatenates an array of `DataFrame` instances into a single new `DataFrame`.

#### `const v = version()`

Returns the underlying Polars version string.

### `Expr`

A lazily-evaluated column expression. Construct expressions with `col`, `lit`, `all`, or `when`, then chain methods. Every method returns a new `Expr` unless noted, so expressions are immutable and composable. Where a method takes another operand it is coerced through the same rules as `lit`, so numbers, strings, booleans, arrays, and `Expr`/`Series` values are all accepted.

#### `expr.alias(name)`

Renames the resulting column to `name`.

#### `expr.cast(dtype)`

Casts the expression to `dtype` (e.g. `'i64'`, `'f64'`, `'str'`).

#### `expr.isNull()`
#### `expr.isNotNull()`

Boolean masks for null / non-null entries.

#### `expr.fillNull(other)`

Replaces null entries with `other`.

#### `expr.over(by)`

Evaluates the expression as a window function partitioned by the column names in the `by` array.

#### `expr.neg()`
#### `expr.not()`

Unary negation and logical NOT.

#### `expr.add(other)`
#### `expr.sub(other)`
#### `expr.mul(other)`
#### `expr.div(other)`
#### `expr.mod(other)`

Element-wise arithmetic against `other`.

#### `expr.eq(other)`
#### `expr.neq(other)`
#### `expr.lt(other)`
#### `expr.ltEq(other)`
#### `expr.gt(other)`
#### `expr.gtEq(other)`

Element-wise comparisons returning a boolean expression.

#### `expr.and(other)`
#### `expr.or(other)`
#### `expr.xor(other)`

Element-wise logical combinators.

#### `expr.sum()`
#### `expr.mean()`
#### `expr.min()`
#### `expr.max()`
#### `expr.median()`
#### `expr.std()`
#### `expr.var()`
#### `expr.count()`
#### `expr.nUnique()`
#### `expr.first()`
#### `expr.last()`

Aggregations that reduce the expression to a single value (per group when used inside `groupBy().agg()` or `over()`).

#### `expr.quantile(q)`

Aggregates to the `q` quantile (`0`–`1`).

#### `expr.abs()`
#### `expr.sign()`
#### `expr.sqrt()`
#### `expr.cbrt()`
#### `expr.log1p()`
#### `expr.exp()`
#### `expr.floor()`
#### `expr.ceil()`

Element-wise math functions.

#### `expr.pow(exponent)`

Raises each element to `exponent`.

#### `expr.log([base])`

Logarithm with the given `base`, defaulting to the natural log (`Math.E`).

#### `expr.round([decimals])`

Rounds to `decimals` decimal places, defaulting to `0`.

#### `expr.clip(min, max)`
#### `expr.clipMin(min)`
#### `expr.clipMax(max)`

Clamps values to the given bounds.

#### `expr.sin()` / `expr.cos()` / `expr.tan()`
#### `expr.asin()` / `expr.acos()` / `expr.atan()`
#### `expr.sinh()` / `expr.cosh()` / `expr.tanh()`

Trigonometric and hyperbolic functions.

#### `expr.shift([n])`

Shifts values by `n` positions (default `1`).

#### `expr.diff([n[, nullBehavior]])`

First discrete difference over a distance of `n` (default `1`). `nullBehavior` is `'ignore'` (default) or `'drop'`.

#### `expr.isIn(values[, nullsEqual])`

Boolean mask of membership in `values`. `nullsEqual` (default `false`) controls whether nulls compare equal.

#### `expr.isBetween(low, high[, closed])`

Boolean mask of values in `[low, high]`. `closed` is `'both'` (default), `'left'`, `'right'`, or `'none'`.

#### `expr.rank([options])`

Ranks values. `options` is passed through to Polars (e.g. ranking method, descending order).

#### `expr.topK(k)`
#### `expr.bottomK(k)`

Returns the `k` largest / smallest values.

#### `expr.str`

Returns a `StrExpr` accessor for string operations on this expression.

#### `expr.dt`

Returns a `DtExpr` accessor for temporal operations on this expression.

### `StrExpr`

String namespace, reached via `expr.str`. Each method returns a new `Expr`.

#### `str.contains(pat[, strict])`

Matches the regex `pat`. `strict` (default `true`) controls whether an invalid pattern throws.

#### `str.containsLiteral(pat)`

Matches the literal substring `pat`.

#### `str.startsWith(pat)`
#### `str.endsWith(pat)`

Prefix / suffix tests.

#### `str.toLowerCase()`
#### `str.toUpperCase()`

Case conversion.

#### `str.lenChars()`
#### `str.lenBytes()`

Length in characters / bytes.

#### `str.replace(pat, value[, literal])`
#### `str.replaceAll(pat, value[, literal])`

Replaces the first / all matches of `pat` with `value`. `literal` (default `false`) treats `pat` as a literal rather than a regex.

#### `str.slice(offset, length)`

Substring starting at `offset` of `length` characters.

#### `str.head(n)`
#### `str.tail(n)`

First / last `n` characters.

#### `str.stripChars(matches)`
#### `str.stripCharsStart(matches)`
#### `str.stripCharsEnd(matches)`

Strips the characters in `matches` from both ends / the start / the end.

#### `str.split(by)`

Splits each string on the separator `by`, producing a list column.

#### `str.padStart(length[, fill])`
#### `str.padEnd(length[, fill])`

Pads to `length` using `fill` (default a single space).

#### `str.zfill(length)`

Left-pads numeric strings with `'0'` to `length`.

#### `str.find(pat[, strict])`

Index of the first regex match of `pat`. `strict` defaults to `true`.

#### `str.countMatches(pat[, literal])`

Counts matches of `pat`. `literal` defaults to `false`.

#### `str.reverse()`

Reverses each string.

#### `str.toDate(format)`

Parses strings into dates using the `strftime` `format`.

### `DtExpr`

Datetime namespace, reached via `expr.dt`. Each method returns a new `Expr`.

#### `dt.year()` / `dt.isoYear()` / `dt.quarter()` / `dt.month()` / `dt.week()` / `dt.weekday()` / `dt.day()` / `dt.ordinalDay()`
#### `dt.hour()` / `dt.minute()` / `dt.second()`

Extract the named temporal component as an integer.

#### `dt.daysInMonth()`

Number of days in the value's month.

#### `dt.monthStart()`
#### `dt.monthEnd()`

Snaps to the first / last day of the month.

#### `dt.strftime(format)`

Formats temporal values as strings using the `strftime` `format`.

#### `dt.timestamp([timeUnit])`

Returns the timestamp as an integer. `timeUnit` is `'us'` (default), `'ms'`, or `'ns'`.

### `Series`

A typed, named one-dimensional array.

#### `const s = new Series(handle)`

Wraps a native series handle. Prefer the static constructors below.

#### `Series.i8(name, values)`
#### `Series.i64(name, values)`
#### `Series.f64(name, values)`
#### `Series.bool(name, values)`
#### `Series.str(name, values)`

Create a `Series` of the given dtype from `name` and an array of `values`.

#### `series.clear()`

Returns an empty `Series` of the same dtype.

#### `series.toDataFrame()`

Wraps the series in a single-column `DataFrame`.

#### `series.toFloat()`

Casts to a floating-point `Series`.

#### `series.print()`

Prints the series to the console.

#### `series.sort([options])`

Returns a sorted `Series`. `options` is passed through to Polars (e.g. descending order).

#### `series.sum()`
#### `series.mean()`
#### `series.min()`
#### `series.max()`
#### `series.median()`
#### `series.std()`
#### `series.var()`

Reduce the series to a single scalar value.

#### `series.quantile(q)`

The `q` quantile (`0`–`1`).

#### `series.len()`

Number of elements.

#### `series.name()`

The series name.

#### `series.rename(name)`

Renames the series in place and returns `this`.

#### `series.dtype()`

The series dtype as a string.

#### `series.cast(dtype)`

Returns the series cast to `dtype`.

#### `series.nullCount()`

Number of null entries.

#### `series.isNull()`
#### `series.isNotNull()`

Boolean `Series` masks for null / non-null entries.

#### `series.dropNulls()`

Returns the series with nulls removed.

#### `series.fillNull(strategy)`

Returns the series with nulls filled per `strategy`.

#### `series.unique()`

Returns the distinct values as a `Series`.

#### `series.nUnique()`

Count of distinct values.

#### `series.head([length])`
#### `series.tail([length])`

First / last `length` elements.

#### `series.slice(offset, length)`

Sub-series of `length` elements starting at `offset`.

#### `series.reverse()`

Returns the series reversed.

#### `series.shift(periods)`

Shifts elements by `periods`.

#### `series.argSort([options])`

Returns the indices that would sort the series as a `Series`.

#### `series.argMin()`
#### `series.argMax()`

Index of the minimum / maximum value.

#### `series.equals(other)`

`true` if `other` (a `Series`) is element-wise equal.

#### `series.get(idx)`

The value at index `idx`.

#### `series.toArray()`

Returns the values as a JavaScript array.

#### `series.valueCounts()`

Returns a `DataFrame` of distinct values and their counts.

### `DataFrame`

A collection of equal-length named `Series` (columns).

#### `const df = new DataFrame(columns)`

Builds a `DataFrame`. `columns` is an array of `Series`, or an existing native handle.

#### `DataFrame.readCSV(path)`
#### `DataFrame.readJSON(path)`

Read a `DataFrame` from a CSV / JSON file at `path`.

#### `df.sort(columns[, options])`

Returns the frame sorted by `columns`. `options` is passed through to Polars (e.g. descending order).

#### `df.groupBy(by)`

Returns a `GroupBy` grouped by the column names in `by`.

#### `df.print()`

Prints the frame to the console.

#### `df.shape()`

Returns `[height, width]`.

#### `df.select(selection)`

Returns a new frame with only the columns named in `selection`.

#### `df.head([length])`
#### `df.tail([length])`

First / last `length` rows.

#### `df.reverse()`

Returns the frame with rows reversed.

#### `df.nullCount()`

Returns a one-row `DataFrame` of null counts per column.

#### `df.height()`
#### `df.width()`
#### `df.size()`

Row count, column count, and total cell count.

#### `df.isEmpty()`

`true` if the frame has no rows.

#### `df.drop(name)`

Returns the frame without column `name`.

#### `df.pop(name)`

Removes column `name` and returns it as a `Series`.

#### `df.dropNulls([subset])`

Returns the frame with rows containing nulls removed, optionally restricted to the columns in `subset`.

#### `df.lazy()`

Returns a `LazyFrame` for building an optimized query.

#### `df.column(name)`

Returns column `name` as a `Series`.

#### `df.columns()`

Returns the column names as an array.

#### `df.rename(column, name)`

Returns the frame with `column` renamed to `name`.

#### `df.replace(column, newCol)`
#### `df.setColumn(column, newCol)`

Returns the frame with `column` replaced by the `Series` `newCol`.

#### `df.replaceColumn(index, newCol)`

Returns the frame with the column at `index` replaced by the `Series` `newCol`.

#### `df.explode(columns)`

Explodes the list-valued `columns`, producing one row per element.

#### `df.apply(name, f)`

Returns the frame with column `name` transformed by `f`, which receives and returns a `Series`.

#### `df.slice(offset, length)`

Sub-frame of `length` rows starting at `offset`.

#### `df.splitAt(offset)`

Splits the frame at `offset`, returning an array of two `DataFrame` instances.

#### `df.shift(periods)`

Shifts rows by `periods`.

#### `df.concat(other)`

Returns a new frame stacking `other`'s rows beneath this frame.

#### `df.concatMut(other)`
#### `df.extend(other)`

Append `other`'s rows in place and return `this`.

#### `df.setColumnNames(names)`

Renames all columns from the `names` array in place and returns `this`.

#### `df.clone()`

Returns a copy of the frame.

#### `df.isUnique()`
#### `df.isDuplicated()`

Boolean `Series` masks marking unique / duplicated rows.

#### `df.minHorizontal()`
#### `df.maxHorizontal()`
#### `df.sumHorizontal()`
#### `df.meanHorizontal()`

Reduce across columns within each row, returning a `Series`.

#### `df.filter(mask)`

Returns the rows where the boolean `Expr`/`Series` `mask` is true.

#### `df.withColumn(series)`

Returns the frame with the `Series` added or replaced.

#### `df.withColumns(seriesArr)`
#### `df.hstack(seriesArr)`

Returns the frame with the array of `Series` added as columns.

#### `df.unique([subset])`

Returns the frame with duplicate rows removed, optionally considering only the columns in `subset`.

#### `df.sample(n[, withReplacement[, shuffle[, seed]]])`

Returns `n` randomly sampled rows. `withReplacement` and `shuffle` default to `false`; `seed` makes sampling deterministic.

#### `df.transpose()`

Returns the transposed frame.

#### `df.sum()`
#### `df.mean()`
#### `df.min()`
#### `df.max()`
#### `df.median()`

Returns a one-row `DataFrame` reducing each column.

#### `df.equals(other)`

`true` if `other` (a `DataFrame`) is equal.

#### `df.estimatedSize()`

Estimated in-memory size in bytes.

#### `df.writeCSV(path)`
#### `df.writeJSON(path)`

Write the frame to `path` and return `this`.

### `LazyFrame`

A lazily-evaluated query plan. Build it up with the methods below and call `collect()` to execute. Created from `df.lazy()`.

#### `lf.collect()`

Executes the query and returns a `DataFrame`.

#### `lf.select(columns)`

Selects columns. `columns` is either an array of `Expr` or an array of column-name strings.

#### `lf.filter(expr)`
#### `lf.filter(column, op, value)`

Filters rows. Pass a single boolean `Expr`, or a `column` name with a comparison operator `op` and `value`.

#### `lf.groupBy(by)`

Returns a `GroupBy` grouped by the column names in `by`.

#### `lf.join(other, leftOn, rightOn[, how])`

Joins `other` (a `LazyFrame`) on `leftOn` / `rightOn`. `how` is `'inner'` (default), `'left'`, `'outer'`, etc.

#### `lf.withColumn(value)`

Adds or replaces a column from an `Expr` or a `Series`.

#### `lf.withColumns(values)`

Adds or replaces columns from an array of `Expr` or `Series`.

#### `lf.sort(columns[, options])`

Sorts by `columns`. `options` is passed through to Polars.

#### `lf.reverse()`

Reverses row order.

#### `lf.head([n])`
#### `lf.tail([n])`

First / last `n` rows.

#### `lf.slice(offset, length)`

Sub-frame of `length` rows starting at `offset`.

#### `lf.unique([subset])`

Removes duplicate rows, optionally restricted to the columns in `subset`.

#### `lf.dropNulls([subset])`

Removes rows containing nulls, optionally restricted to the columns in `subset`.

#### `lf.rename(from, to)`

Renames column `from` to `to`.

#### `lf.drop(columns)`

Removes the named `columns`.

#### `lf.explode(columns)`

Explodes the list-valued `columns`.

#### `lf.clone()`

Returns a copy of the query plan.

#### `lf.count()`
#### `lf.sum()`
#### `lf.mean()`
#### `lf.median()`
#### `lf.min()`
#### `lf.max()`
#### `lf.std()`
#### `lf.var()`
#### `lf.first()`
#### `lf.last()`

Frame-wide reductions, returning a `LazyFrame`.

#### `lf.quantile(q)`

Frame-wide `q` quantile, returning a `LazyFrame`.

#### `lf.explain([optimized])`

Returns the query plan as a string. `optimized` (default `true`) shows the optimized plan.

### `GroupBy`

A grouped view produced by `df.groupBy(by)` or `lf.groupBy(by)`. Each method returns a `LazyFrame`.

#### `group.agg(exprs)`

Aggregates each group using the array of `Expr` in `exprs`.

#### `group.sum()`
#### `group.mean()`
#### `group.min()`
#### `group.max()`
#### `group.median()`
#### `group.count()`
#### `group.first()`
#### `group.last()`
#### `group.std()`
#### `group.var()`

Apply the named aggregation to every non-key column of each group.

#### `group.quantile(q)`

Per-group `q` quantile.

#### `group.head(n)`
#### `group.tail(n)`

First / last `n` rows of each group.

## License

Apache-2.0
