// Simplest possible walkthrough. Run with: bare examples/simple.js

const { DataFrame, Series, col } = require('..')

// Build a frame from three Series.
const df = new DataFrame([
  Series.str('name', ['Aurora', 'Felix', 'Mateo', 'Noor']),
  Series.i64('age', [34, 31, 14, 9]),
  Series.i64('height', [179, 184, 181, 135])
])

// 1. Print the whole frame.
df.print()

// 2. A quick stat on one column.
console.log('mean height:', df.column('height').mean())

// 3. Filter via the Expr DSL.
df.lazy().filter(col('age').gtEq(18)).collect().print()

// 4. Group + aggregate in one chain.
df.lazy()
  .withColumn(col('age').gtEq(18).alias('is_adult'))
  .groupBy(['is_adult'])
  .agg([col('height').max().alias('tallest')])
  .collect()
  .sort(['is_adult'])
  .print()
