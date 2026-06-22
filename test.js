const test = require('brittle')
const fs = require('bare-fs')
const path = require('bare-path')
const {
  DataFrame,
  Series,
  LazyFrame,
  col,
  lit,
  when,
  concat,
  version
} = require('.')

function tmpPath(suffix) {
  return path.join(
    '/tmp',
    `bare-polars-${Date.now()}-${Math.random().toString(36).slice(2)}${suffix}`
  )
}

// ── Module surface ─────────────────────────────────────────────────────
test('module exports core classes and helpers', (t) => {
  t.ok(DataFrame)
  t.ok(Series)
  t.ok(LazyFrame)
  t.is(typeof col, 'function')
  t.is(typeof lit, 'function')
  t.is(typeof when, 'function')
  t.is(typeof concat, 'function')
  t.is(typeof version, 'function')
  t.ok(/polars/.test(version()))
})

// ── Series construction & metadata ─────────────────────────────────────
test('Series: constructors and metadata', (t) => {
  const a = Series.i64('a', [1, 2, 3])
  t.is(a.name(), 'a')
  t.is(a.len(), 3)
  t.is(a.dtype(), 'i64')

  const b = Series.f64('b', [1.5, 2.5])
  t.is(b.dtype(), 'f64')

  const c = Series.str('c', ['x', 'y'])
  t.is(c.dtype(), 'str')

  const d = Series.bool('d', [true, false, true])
  t.is(d.dtype(), 'bool')
})

test('Series: rename mutates name', (t) => {
  const s = Series.i64('old', [1, 2])
  s.rename('new')
  t.is(s.name(), 'new')
})

test('Series: aggregations', (t) => {
  const s = Series.i64('x', [1, 2, 3, 4, 5])
  t.is(s.sum(), 15)
  t.is(s.mean(), 3)
  t.is(s.min(), 1)
  t.is(s.max(), 5)
  t.is(s.median(), 3)
  t.is(s.nUnique(), 5)
  t.is(s.len(), 5)
  t.ok(s.std() > 0)
  t.ok(s.var() > 0)
})

test('Series: toArray roundtrip', (t) => {
  t.alike(Series.i64('i', [1, 2, 3]).toArray(), [1, 2, 3])
  t.alike(Series.str('s', ['a', 'b']).toArray(), ['a', 'b'])
  t.alike(Series.bool('b', [true, false]).toArray(), [true, false])
})

test('Series: get by index', (t) => {
  const s = Series.i64('x', [10, 20, 30])
  t.is(s.get(0), 10)
  t.is(s.get(2), 30)
})

test('Series: cast', (t) => {
  const s = Series.i64('x', [1, 2, 3])
  const f = s.cast('f64')
  t.is(f.dtype(), 'f64')
  t.alike(f.toArray(), [1, 2, 3])
})

test('Series: head/tail/slice/reverse', (t) => {
  const s = Series.i64('x', [1, 2, 3, 4, 5])
  t.alike(s.head(2).toArray(), [1, 2])
  t.alike(s.tail(2).toArray(), [4, 5])
  t.alike(s.slice(1, 3).toArray(), [2, 3, 4])
  t.alike(s.reverse().toArray(), [5, 4, 3, 2, 1])
})

test('Series: nulls', (t) => {
  // Build a series with nulls via cast-from-floats trick: easier to test via DataFrame
  const df = DataFrame.readCSV('./test/data.csv')
  const ages = df.column('age')
  t.is(ages.nullCount(), 2)
  t.is(ages.isNull().sum(), 2)
  t.is(ages.isNotNull().sum(), 6)
  t.is(ages.dropNulls().len(), 6)
})

test('Series: unique and valueCounts', (t) => {
  const s = Series.i64('x', [1, 1, 2, 2, 3])
  t.is(s.nUnique(), 3)
  const u = s.unique().sort()
  t.alike(u.toArray(), [1, 2, 3])
  const vc = s.valueCounts()
  t.is(vc.height(), 3)
  t.alike(vc.columns().sort(), ['count', 'x'])
})

test('Series: sort with options object', (t) => {
  const s = Series.i64('x', [3, 1, 2])
  t.alike(s.sort().toArray(), [1, 2, 3])
  t.alike(s.sort({ descending: true }).toArray(), [3, 2, 1])
})

// ── DataFrame core ─────────────────────────────────────────────────────
test('DataFrame: construct from Series', (t) => {
  const df = new DataFrame([
    Series.i64('a', [1, 2, 3]),
    Series.str('b', ['x', 'y', 'z'])
  ])
  t.alike(df.shape(), [3, 2])
  t.is(df.width(), 2)
  t.is(df.height(), 3)
  t.is(df.size(), 6)
  t.is(df.isEmpty(), false)
  t.alike(df.columns(), ['a', 'b'])
})

test('DataFrame: column returns a Series', (t) => {
  const df = new DataFrame([Series.i64('x', [10, 20, 30])])
  const col_x = df.column('x')
  t.ok(col_x instanceof Series)
  t.alike(col_x.toArray(), [10, 20, 30])
})

test('DataFrame: select / drop / rename', (t) => {
  const df = new DataFrame([
    Series.i64('a', [1, 2]),
    Series.i64('b', [3, 4]),
    Series.i64('c', [5, 6])
  ])
  t.alike(df.select(['a', 'c']).columns(), ['a', 'c'])
  t.alike(df.drop('b').columns(), ['a', 'c'])
  t.alike(df.rename('b', 'B').columns(), ['a', 'B', 'c'])
})

test('DataFrame: pop returns the dropped column', (t) => {
  const df = new DataFrame([
    Series.i64('a', [1, 2]),
    Series.str('b', ['x', 'y'])
  ])
  const popped = df.pop('b')
  t.ok(popped instanceof Series)
  t.is(popped.name(), 'b')
  t.alike(df.columns(), ['a']) // pop is in-place
})

test('DataFrame: head/tail/slice/reverse', (t) => {
  const df = new DataFrame([Series.i64('x', [1, 2, 3, 4, 5])])
  t.is(df.head(2).height(), 2)
  t.is(df.tail(2).height(), 2)
  t.is(df.slice(1, 3).height(), 3)
  t.alike(df.reverse().column('x').toArray(), [5, 4, 3, 2, 1])
})

test('DataFrame: withColumn / withColumns / hstack', (t) => {
  const df = new DataFrame([Series.i64('a', [1, 2])])
  const df2 = df.withColumn(Series.i64('b', [3, 4]))
  t.alike(df2.columns(), ['a', 'b'])

  const df3 = df.withColumns([Series.i64('b', [3, 4]), Series.i64('c', [5, 6])])
  t.alike(df3.columns(), ['a', 'b', 'c'])

  const df4 = df.hstack([Series.i64('d', [7, 8])])
  t.alike(df4.columns(), ['a', 'd'])
})

test('DataFrame: setColumn (upsert)', (t) => {
  const df = new DataFrame([Series.i64('a', [1, 2])])
  const df2 = df.setColumn('a', Series.i64('a', [10, 20]))
  t.alike(df2.column('a').toArray(), [10, 20])
  const df3 = df.setColumn('b', Series.i64('b', [3, 4]))
  t.alike(df3.columns(), ['a', 'b'])
})

test('DataFrame: filter via boolean Series', (t) => {
  const df = new DataFrame([
    Series.i64('age', [10, 20, 30]),
    Series.str('name', ['a', 'b', 'c'])
  ])
  const mask = Series.bool('m', [false, true, true])
  const filtered = df.filter(mask)
  t.is(filtered.height(), 2)
  t.alike(filtered.column('age').toArray(), [20, 30])
})

test('DataFrame: dropNulls (with and without subset)', (t) => {
  const df = DataFrame.readCSV('./test/data.csv')
  t.is(df.height(), 8)
  t.is(df.dropNulls().height(), 6)
  t.is(df.dropNulls(['age']).height(), 6)
})

test('DataFrame: concat', (t) => {
  const a = new DataFrame([Series.i64('x', [1, 2])])
  const b = new DataFrame([Series.i64('x', [3, 4])])
  const c = concat([a, b])
  t.alike(c.column('x').toArray(), [1, 2, 3, 4])
})

test('DataFrame: unique / equals / clone', (t) => {
  const df = new DataFrame([Series.i64('x', [1, 1, 2, 3, 3])])
  const u = df.unique().sort(['x'])
  t.alike(u.column('x').toArray(), [1, 2, 3])

  const a = new DataFrame([Series.i64('x', [1, 2])])
  const b = new DataFrame([Series.i64('x', [1, 2])])
  t.is(a.equals(b), true)

  const cloned = a.clone()
  t.is(a.equals(cloned), true)
})

test('DataFrame: reductions return single-row frames', (t) => {
  const df = new DataFrame([
    Series.i64('a', [1, 2, 3]),
    Series.i64('b', [10, 20, 30])
  ])
  const sum = df.sum()
  t.is(sum.height(), 1)
  t.is(sum.column('a').get(0), 6)
  t.is(sum.column('b').get(0), 60)

  const mean = df.mean()
  t.is(mean.column('a').get(0), 2)
})

test('DataFrame: horizontal reductions', (t) => {
  const df = new DataFrame([
    Series.i64('a', [1, 2, 3]),
    Series.i64('b', [10, 20, 30])
  ])
  t.alike(df.sumHorizontal().toArray(), [11, 22, 33])
  t.alike(df.meanHorizontal().toArray(), [5.5, 11, 16.5])
})

test('DataFrame: sort with options object', (t) => {
  const df = new DataFrame([
    Series.i64('age', [30, 10, 20]),
    Series.str('name', ['c', 'a', 'b'])
  ])
  const sorted = df.sort(['age'])
  t.alike(sorted.column('age').toArray(), [10, 20, 30])

  const sortedDesc = df.sort(['age'], { descending: [true] })
  t.alike(sortedDesc.column('age').toArray(), [30, 20, 10])
})

// ── LazyFrame ──────────────────────────────────────────────────────────
test('LazyFrame: roundtrip via lazy().collect()', (t) => {
  const df = new DataFrame([Series.i64('x', [1, 2, 3])])
  const back = df.lazy().collect()
  t.is(df.equals(back), true)
})

test('LazyFrame: select with column names', (t) => {
  const df = new DataFrame([Series.i64('a', [1, 2]), Series.i64('b', [3, 4])])
  const out = df.lazy().select(['a']).collect()
  t.alike(out.columns(), ['a'])
})

test('LazyFrame: sort / head / tail / reverse / slice', (t) => {
  const df = new DataFrame([Series.i64('x', [5, 1, 3, 2, 4])])
  t.alike(
    df.lazy().sort(['x']).collect().column('x').toArray(),
    [1, 2, 3, 4, 5]
  )
  t.is(df.lazy().head(2).collect().height(), 2)
  t.is(df.lazy().tail(2).collect().height(), 2)
  t.is(df.lazy().slice(1, 2).collect().height(), 2)
  t.alike(df.lazy().reverse().collect().column('x').toArray(), [4, 2, 3, 1, 5])
})

test('LazyFrame: rename / drop', (t) => {
  const df = new DataFrame([Series.i64('a', [1, 2]), Series.i64('b', [3, 4])])
  const renamed = df.lazy().rename('a', 'A').collect()
  t.alike(renamed.columns(), ['A', 'b'])

  const dropped = df.lazy().drop(['b']).collect()
  t.alike(dropped.columns(), ['a'])
})

test('LazyFrame: explain returns a plan string', (t) => {
  const df = new DataFrame([Series.i64('x', [1, 2])])
  const plan = df
    .lazy()
    .select([col('x').sum()])
    .explain()
  t.is(typeof plan, 'string')
  t.ok(plan.length > 0)
})

// ── Expr DSL ───────────────────────────────────────────────────────────
test('Expr: filter via col().gt()', (t) => {
  const df = new DataFrame([Series.i64('age', [5, 15, 25, 35])])
  const out = df.lazy().filter(col('age').gt(10)).collect()
  t.alike(out.column('age').toArray(), [15, 25, 35])
})

test('Expr: arithmetic and alias', (t) => {
  const df = new DataFrame([
    Series.i64('a', [1, 2, 3]),
    Series.i64('b', [10, 20, 30])
  ])
  const out = df
    .lazy()
    .withColumn(col('a').add(col('b')).alias('sum_ab'))
    .collect()
  t.alike(out.column('sum_ab').toArray(), [11, 22, 33])
})

test('Expr: comparison + logical combinators', (t) => {
  const df = new DataFrame([
    Series.i64('a', [1, 2, 3, 4]),
    Series.i64('b', [10, 20, 30, 40])
  ])
  const out = df
    .lazy()
    .filter(col('a').gtEq(2).and(col('b').lt(40)))
    .collect()
  t.alike(out.column('a').toArray(), [2, 3])
})

test('Expr: when/then/otherwise', (t) => {
  const df = new DataFrame([Series.i64('age', [5, 15, 25])])
  const out = df
    .lazy()
    .withColumn(
      when(col('age').gtEq(18))
        .then(lit('adult'))
        .when(col('age').gtEq(13))
        .then(lit('teen'))
        .otherwise(lit('child'))
        .alias('bucket')
    )
    .collect()
  t.alike(out.column('bucket').toArray(), ['child', 'teen', 'adult'])
})

test('Expr: cast', (t) => {
  const df = new DataFrame([Series.i64('x', [1, 2, 3])])
  const out = df.lazy().withColumn(col('x').cast('f64').alias('xf')).collect()
  t.is(out.column('xf').dtype(), 'f64')
})

test('Expr: aggregations inside select', (t) => {
  const df = new DataFrame([Series.i64('x', [1, 2, 3, 4, 5])])
  const out = df
    .lazy()
    .select([
      col('x').sum().alias('s'),
      col('x').mean().alias('m'),
      col('x').min().alias('mn'),
      col('x').max().alias('mx'),
      col('x').count().alias('c')
    ])
    .collect()
  t.is(out.column('s').get(0), 15)
  t.is(out.column('m').get(0), 3)
  t.is(out.column('mn').get(0), 1)
  t.is(out.column('mx').get(0), 5)
  t.is(out.column('c').get(0), 5)
})

// ── GroupBy ────────────────────────────────────────────────────────────
test('GroupBy: agg with expressions', (t) => {
  const df = new DataFrame([
    Series.str('g', ['a', 'a', 'b', 'b', 'b']),
    Series.i64('v', [1, 2, 10, 20, 30])
  ])
  const out = df
    .groupBy(['g'])
    .agg([col('v').sum().alias('sum_v'), col('v').count().alias('n')])
    .collect()
    .sort(['g'])
  t.alike(out.column('g').toArray(), ['a', 'b'])
  t.alike(out.column('sum_v').toArray(), [3, 60])
  t.alike(out.column('n').toArray(), [2, 3])
})

test('GroupBy: shorthand sum / mean', (t) => {
  const df = new DataFrame([
    Series.str('g', ['a', 'a', 'b']),
    Series.i64('v', [10, 20, 30])
  ])
  const sums = df.groupBy(['g']).sum().collect().sort(['g'])
  t.alike(sums.column('g').toArray(), ['a', 'b'])
  t.alike(sums.column('v').toArray(), [30, 30])
})

// ── IO roundtrip ───────────────────────────────────────────────────────
test('IO: CSV roundtrip', (t) => {
  const orig = new DataFrame([
    Series.i64('a', [1, 2, 3]),
    Series.str('b', ['x', 'y', 'z'])
  ])
  const p = tmpPath('.csv')
  orig.writeCSV(p)
  const reread = DataFrame.readCSV(p)
  t.is(orig.equals(reread), true)
  fs.unlinkSync(p)
})

test('IO: JSON roundtrip', (t) => {
  const orig = new DataFrame([Series.i64('a', [1, 2, 3])])
  const p = tmpPath('.json')
  orig.writeJSON(p)
  const reread = DataFrame.readJSON(p)
  t.is(orig.equals(reread), true)
  fs.unlinkSync(p)
})

// ── Error handling ─────────────────────────────────────────────────────
test('errors: missing column throws', (t) => {
  const df = new DataFrame([Series.i64('a', [1])])
  t.exception(() => df.column('typo'), /not found/)
})

test('errors: missing CSV path throws', (t) => {
  t.exception(() => DataFrame.readCSV('/does/not/exist.csv'))
})

test('errors: bad cast throws', (t) => {
  t.exception(
    () => Series.str('s', ['x']).cast('not_a_dtype'),
    /unsupported dtype/
  )
})

test('errors: bad fillNull strategy throws', (t) => {
  t.exception(
    () => Series.i64('s', [1]).fillNull('bogus'),
    /unsupported fillNull/
  )
})

test('errors: schema mismatch on concat throws', (t) => {
  const a = new DataFrame([Series.i64('x', [1])])
  const b = new DataFrame([Series.str('x', ['a'])])
  t.exception(() => concat([a, b]).column('x'))
})

// ── DataFrame.apply ────────────────────────────────────────────────────
test('apply: callback transforms the column', (t) => {
  const df = new DataFrame([Series.i64('x', [1, 2, 3])])
  const out = df.apply('x', (s) => s.cast('f64'))
  t.is(out.column('x').dtype(), 'f64')
  t.alike(out.column('x').toArray(), [1, 2, 3])
})

test('apply: callback throwing propagates the error', (t) => {
  const df = new DataFrame([Series.i64('x', [1, 2, 3])])
  t.exception(
    () =>
      df.apply('x', () => {
        throw new Error('boom from JS')
      }),
    /boom from JS/
  )
})

test('apply: callback throwing does not crash subsequent calls', (t) => {
  const df = new DataFrame([Series.i64('x', [1, 2, 3])])
  try {
    df.apply('x', () => {
      throw new Error('first failure')
    })
  } catch (_) {
    // expected
  }
  // The process is still alive and apply can be called again with a good callback.
  const ok = df.apply('x', (s) => s.cast('f64'))
  t.is(ok.column('x').dtype(), 'f64')
})

test('apply: missing column name throws', (t) => {
  const df = new DataFrame([Series.i64('x', [1])])
  t.exception(() => df.apply('typo', (s) => s), /not found/)
})

// ── Expr.str accessor ──────────────────────────────────────────────────
test('Expr.str: contains / startsWith / endsWith', (t) => {
  const df = new DataFrame([Series.str('name', ['hello', 'world', 'help'])])
  const out = df
    .lazy()
    .select([
      col('name').str.contains('hel').alias('has_hel'),
      col('name').str.startsWith('h').alias('starts_h'),
      col('name').str.endsWith('lp').alias('ends_lp')
    ])
    .collect()
  t.alike(out.column('has_hel').toArray(), [true, false, true])
  t.alike(out.column('starts_h').toArray(), [true, false, true])
  t.alike(out.column('ends_lp').toArray(), [false, false, true])
})

test('Expr.str: case conversions', (t) => {
  const df = new DataFrame([Series.str('s', ['Hello', 'world'])])
  const out = df
    .lazy()
    .select([
      col('s').str.toLowerCase().alias('lo'),
      col('s').str.toUpperCase().alias('up')
    ])
    .collect()
  t.alike(out.column('lo').toArray(), ['hello', 'world'])
  t.alike(out.column('up').toArray(), ['HELLO', 'WORLD'])
})

test('Expr.str: lenChars / lenBytes', (t) => {
  const df = new DataFrame([Series.str('s', ['ab', 'cdef'])])
  const out = df
    .lazy()
    .select([
      col('s').str.lenChars().alias('lc'),
      col('s').str.lenBytes().alias('lb')
    ])
    .collect()
  t.alike(out.column('lc').toArray(), [2, 4])
  t.alike(out.column('lb').toArray(), [2, 4])
})

test('Expr.str: replace / replaceAll', (t) => {
  const df = new DataFrame([Series.str('s', ['foo bar foo'])])
  const out = df
    .lazy()
    .select([
      col('s').str.replace('foo', 'baz', true).alias('r1'),
      col('s').str.replaceAll('foo', 'baz', true).alias('rN')
    ])
    .collect()
  t.alike(out.column('r1').toArray(), ['baz bar foo'])
  t.alike(out.column('rN').toArray(), ['baz bar baz'])
})

test('Expr.str: padStart / padEnd / zfill', (t) => {
  const df = new DataFrame([Series.str('s', ['7', '42'])])
  const out = df
    .lazy()
    .select([
      col('s').str.padStart(3, '0').alias('ps'),
      col('s').str.padEnd(3, '_').alias('pe'),
      col('s').str.zfill(3).alias('z')
    ])
    .collect()
  t.alike(out.column('ps').toArray(), ['007', '042'])
  t.alike(out.column('pe').toArray(), ['7__', '42_'])
  t.alike(out.column('z').toArray(), ['007', '042'])
})

test('Expr.str: reverse', (t) => {
  const df = new DataFrame([Series.str('s', ['abc', 'hello'])])
  const out = df
    .lazy()
    .select([col('s').str.reverse().alias('r')])
    .collect()
  t.alike(out.column('r').toArray(), ['cba', 'olleh'])
})

test('Expr.str: countMatches', (t) => {
  const df = new DataFrame([Series.str('s', ['aaa', 'aba', 'b'])])
  const out = df
    .lazy()
    .select([col('s').str.countMatches('a', true).alias('n')])
    .collect()
  t.alike(out.column('n').toArray(), [3, 2, 0])
})

test('Expr.str: countMatches in groupBy chain', (t) => {
  const df = new DataFrame([
    Series.str('g', ['x', 'x', 'y']),
    Series.str('v', ['hello', 'world', 'help'])
  ])
  const out = df
    .groupBy(['g'])
    .agg([col('v').str.lenChars().sum().alias('total_chars')])
    .collect()
    .sort(['g'])
  t.alike(out.column('g').toArray(), ['x', 'y'])
  t.alike(out.column('total_chars').toArray(), [10, 4]) // hello+world=10, help=4
})

// ── Expr.dt accessor ───────────────────────────────────────────────────
test('Expr.dt: year/month/day from parsed dates', (t) => {
  const df = new DataFrame([Series.str('d', ['2024-03-15', '2025-12-31'])])
  const dated = df
    .lazy()
    .withColumn(col('d').str.toDate('%Y-%m-%d').alias('date'))
  const out = dated
    .select([
      col('date').dt.year().alias('y'),
      col('date').dt.month().alias('m'),
      col('date').dt.day().alias('day')
    ])
    .collect()
  t.alike(out.column('y').toArray(), [2024, 2025])
  t.alike(out.column('m').toArray(), [3, 12])
  t.alike(out.column('day').toArray(), [15, 31])
})

test('Expr.dt: weekday / quarter / ordinalDay', (t) => {
  // 2024-03-15 was a Friday (ISO weekday 5), quarter 1, ordinal day 75.
  const df = new DataFrame([Series.str('d', ['2024-03-15'])])
  const out = df
    .lazy()
    .withColumn(col('d').str.toDate('%Y-%m-%d').alias('date'))
    .select([
      col('date').dt.weekday().alias('wd'),
      col('date').dt.quarter().alias('q'),
      col('date').dt.ordinalDay().alias('od')
    ])
    .collect()
  t.is(out.column('wd').get(0), 5)
  t.is(out.column('q').get(0), 1)
  t.is(out.column('od').get(0), 75)
})

test('Expr.dt: monthStart / monthEnd', (t) => {
  const df = new DataFrame([Series.str('d', ['2024-03-15'])])
  const out = df
    .lazy()
    .withColumn(col('d').str.toDate('%Y-%m-%d').alias('date'))
    .select([
      col('date').dt.monthStart().dt.day().alias('start_day'),
      col('date').dt.monthEnd().dt.day().alias('end_day')
    ])
    .collect()
  t.is(out.column('start_day').get(0), 1)
  t.is(out.column('end_day').get(0), 31)
})

test('Expr.dt: strftime back to string', (t) => {
  const df = new DataFrame([Series.str('d', ['2024-03-15'])])
  const out = df
    .lazy()
    .withColumn(col('d').str.toDate('%Y-%m-%d').alias('date'))
    .select([col('date').dt.strftime('%Y/%m/%d').alias('s')])
    .collect()
  t.alike(out.column('s').toArray(), ['2024/03/15'])
})

test('Expr.dt: bad time unit throws', (t) => {
  const df = new DataFrame([Series.str('d', ['2024-01-01'])])
  t.exception(
    () =>
      df
        .lazy()
        .withColumn(col('d').str.toDate('%Y-%m-%d').alias('date'))
        .select([col('date').dt.timestamp('weeks')])
        .collect(),
    /unsupported time unit/
  )
})

// ── Expr math / numerical ──────────────────────────────────────────────
test('Expr.abs / sign', (t) => {
  const df = new DataFrame([Series.i64('x', [-3, -1, 0, 2, 5])])
  const out = df
    .lazy()
    .select([col('x').abs().alias('a'), col('x').sign().alias('s')])
    .collect()
  t.alike(out.column('a').toArray(), [3, 1, 0, 2, 5])
  t.alike(out.column('s').toArray(), [-1, -1, 0, 1, 1])
})

test('Expr.sqrt / cbrt / pow', (t) => {
  const df = new DataFrame([Series.f64('x', [4.0, 9.0, 27.0])])
  const out = df
    .lazy()
    .select([
      col('x').sqrt().alias('sq'),
      col('x').cbrt().alias('cb'),
      col('x').pow(2).alias('p2')
    ])
    .collect()
  const sq = out.column('sq').toArray()
  t.is(Math.round(sq[0]), 2)
  t.is(Math.round(sq[1]), 3)
  const cb = out.column('cb').toArray()
  t.is(Math.round(cb[2]), 3)
  t.alike(out.column('p2').toArray(), [16, 81, 729])
})

test('Expr.log / log1p / exp', (t) => {
  const df = new DataFrame([Series.f64('x', [1.0, Math.E, Math.E * Math.E])])
  const out = df
    .lazy()
    .select([
      col('x').log().alias('ln'), // natural log (default base = e)
      col('x').log(10).alias('lg'),
      col('x').exp().alias('e_x')
    ])
    .collect()
  const ln = out.column('ln').toArray()
  t.is(Math.round(ln[0]), 0)
  t.is(Math.round(ln[1]), 1)
  t.is(Math.round(ln[2]), 2)

  // log10(1) ≈ 0, log10(e²) ≈ 0.868
  const lg = out.column('lg').toArray()
  t.is(Math.round(lg[0]), 0)
  t.ok(Math.abs(lg[2] - 2 / Math.LN10) < 1e-9)

  // exp(1) ≈ e
  t.ok(Math.abs(out.column('e_x').toArray()[0] - Math.E) < 1e-9)
})

test('Expr.floor / ceil / round', (t) => {
  const df = new DataFrame([Series.f64('x', [1.2, 1.7, -0.5, 3.14159])])
  const out = df
    .lazy()
    .select([
      col('x').floor().alias('fl'),
      col('x').ceil().alias('cl'),
      col('x').round(0).alias('r0'),
      col('x').round(2).alias('r2')
    ])
    .collect()
  // normalize -0 → 0 so brittle's Object.is-based alike is happy
  const norm = (arr) => arr.map((v) => v + 0)
  t.alike(norm(out.column('fl').toArray()), [1, 1, -1, 3])
  t.alike(norm(out.column('cl').toArray()), [2, 2, 0, 4])
  t.alike(norm(out.column('r0').toArray()), [1, 2, -1, 3])
  t.alike(norm(out.column('r2').toArray()), [1.2, 1.7, -0.5, 3.14])
})

test('Expr.clip / clipMin / clipMax', (t) => {
  const df = new DataFrame([Series.i64('x', [-5, 0, 5, 10, 15])])
  const out = df
    .lazy()
    .select([
      col('x').clip(0, 10).alias('c'),
      col('x').clipMin(0).alias('cmin'),
      col('x').clipMax(10).alias('cmax')
    ])
    .collect()
  t.alike(out.column('c').toArray(), [0, 0, 5, 10, 10])
  t.alike(out.column('cmin').toArray(), [0, 0, 5, 10, 15])
  t.alike(out.column('cmax').toArray(), [-5, 0, 5, 10, 10])
})

test('Expr trigonometry: sin / cos / tan + inverse', (t) => {
  const df = new DataFrame([Series.f64('x', [0.0, Math.PI / 2, Math.PI])])
  const out = df
    .lazy()
    .select([
      col('x').sin().alias('s'),
      col('x').cos().alias('c'),
      col('x').sin().asin().alias('s2'),
      col('x').cos().acos().alias('c2')
    ])
    .collect()
  const s = out.column('s').toArray()
  t.ok(Math.abs(s[0] - 0) < 1e-9)
  t.ok(Math.abs(s[1] - 1) < 1e-9)
  const c = out.column('c').toArray()
  t.ok(Math.abs(c[0] - 1) < 1e-9)
  t.ok(Math.abs(c[2] - -1) < 1e-9)
  // asin(sin(0)) = 0
  t.ok(Math.abs(out.column('s2').toArray()[0]) < 1e-9)
})

test('Expr.shift', (t) => {
  const df = new DataFrame([Series.i64('x', [10, 20, 30, 40])])
  const out = df
    .lazy()
    .select([col('x').shift(1).alias('s1'), col('x').shift(-1).alias('sm1')])
    .collect()
  // forward shift of 1 → first value is null, then [10, 20, 30]
  const s1 = out.column('s1').toArray()
  t.is(s1[0], null)
  t.alike(s1.slice(1), [10, 20, 30])
  // back shift → last is null
  const sm1 = out.column('sm1').toArray()
  t.alike(sm1.slice(0, 3), [20, 30, 40])
  t.is(sm1[3], null)
})

test('Expr.diff', (t) => {
  const df = new DataFrame([Series.i64('x', [1, 3, 6, 10])])
  const out = df
    .lazy()
    .select([col('x').diff().alias('d')])
    .collect()
  const d = out.column('d').toArray()
  t.is(d[0], null) // first element has no predecessor
  t.alike(d.slice(1), [2, 3, 4])
})

test('Expr.diff: drop nullBehavior shortens the result', (t) => {
  const df = new DataFrame([Series.i64('x', [1, 3, 6, 10])])
  const out = df
    .lazy()
    .select([col('x').diff(1, 'drop').alias('d')])
    .collect()
  // With "drop", the leading null is removed → 3 rows.
  t.is(out.height(), 3)
  t.alike(out.column('d').toArray(), [2, 3, 4])
})

test('Expr.isIn with array literal', (t) => {
  const df = new DataFrame([Series.i64('x', [1, 2, 3, 4, 5])])
  const out = df
    .lazy()
    .select([col('x').isIn([2, 4]).alias('in')])
    .collect()
  t.alike(out.column('in').toArray(), [false, true, false, true, false])
})

test('Expr.isIn with Series', (t) => {
  const df = new DataFrame([Series.str('x', ['a', 'b', 'c', 'd'])])
  const out = df
    .lazy()
    .select([col('x').isIn(Series.str('vals', ['b', 'd'])).alias('in')])
    .collect()
  t.alike(out.column('in').toArray(), [false, true, false, true])
})

test('Expr.isBetween: closed intervals', (t) => {
  const df = new DataFrame([Series.i64('x', [1, 2, 3, 4, 5])])
  const out = df
    .lazy()
    .select([
      col('x').isBetween(2, 4).alias('both'),
      col('x').isBetween(2, 4, 'left').alias('left'),
      col('x').isBetween(2, 4, 'right').alias('right'),
      col('x').isBetween(2, 4, 'none').alias('none')
    ])
    .collect()
  t.alike(out.column('both').toArray(), [false, true, true, true, false])
  t.alike(out.column('left').toArray(), [false, true, true, false, false])
  t.alike(out.column('right').toArray(), [false, false, true, true, false])
  t.alike(out.column('none').toArray(), [false, false, true, false, false])
})

test('Expr.rank: default (dense, ascending)', (t) => {
  const df = new DataFrame([Series.i64('x', [10, 30, 20, 30, 10])])
  const out = df.lazy().select([col('x').rank().alias('r')]).collect()
  // dense ranking ascending: 10→1, 20→2, 30→3
  t.alike(out.column('r').toArray(), [1, 3, 2, 3, 1])
})

test('Expr.rank: descending + ordinal', (t) => {
  const df = new DataFrame([Series.i64('x', [10, 30, 20, 30, 10])])
  const out = df
    .lazy()
    .select([col('x').rank({ method: 'ordinal', descending: true }).alias('r')])
    .collect()
  // ordinal descending: largest gets rank 1, ties broken by appearance
  const r = out.column('r').toArray()
  t.is(r[1], 1) // first 30
  t.is(r[3], 2) // second 30
  t.is(r[2], 3) // 20
})

test('Expr.topK / bottomK', (t) => {
  const df = new DataFrame([Series.i64('x', [5, 1, 9, 3, 7])])
  const topOut = df
    .lazy()
    .select([col('x').topK(3).alias('top')])
    .collect()
  const top = topOut
    .column('top')
    .toArray()
    .sort((a, b) => b - a)
  t.alike(top, [9, 7, 5])

  const botOut = df
    .lazy()
    .select([col('x').bottomK(2).alias('bot')])
    .collect()
  const bot = botOut
    .column('bot')
    .toArray()
    .sort((a, b) => a - b)
  t.alike(bot, [1, 3])
})

test('Expr ordering: chained filter + topK', (t) => {
  // "give me the top 2 even numbers"
  const df = new DataFrame([Series.i64('x', [4, 1, 8, 2, 5, 6, 3, 10])])
  const out = df
    .lazy()
    .filter(col('x').mod(2).eq(0))
    .select([col('x').topK(2).alias('top')])
    .collect()
  const top = out.column('top').toArray().sort((a, b) => b - a)
  t.alike(top, [10, 8])
})

test('Expr math: chained in groupBy.agg', (t) => {
  const df = new DataFrame([
    Series.str('g', ['a', 'a', 'b', 'b']),
    Series.i64('v', [-4, 16, -9, 25])
  ])
  // For each group, take sqrt(abs(v)) and sum it
  const out = df
    .groupBy(['g'])
    .agg([col('v').abs().sqrt().sum().alias('rooted_sum')])
    .collect()
    .sort(['g'])
  t.alike(out.column('g').toArray(), ['a', 'b'])
  t.alike(out.column('rooted_sum').toArray(), [6, 8]) // sqrt(4)+sqrt(16)=6, sqrt(9)+sqrt(25)=8
})
