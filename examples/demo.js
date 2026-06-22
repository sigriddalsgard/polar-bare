// Smart demo of the polar-bare Polars bindings.
// Run with: bare examples/demo.js
//
// The story: a tiny HR table — 10 employees across 3 departments with
// salaries and join dates. We exercise the Expr DSL, window functions,
// .str / .dt accessors, joins, IO roundtrip, and error handling.

const { DataFrame, Series, col, lit, when, version } = require('..')

function section(title) {
  console.log('')
  console.log('── ' + title + ' ' + '─'.repeat(Math.max(0, 64 - title.length)))
}

// ── seed data ───────────────────────────────────────────────────────────
const employees = new DataFrame([
  Series.i64('id', [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
  Series.str('name', [
    'Aurora',
    'Felix',
    'Noor',
    'Mateo',
    'Priya',
    'Quinn',
    'Theo',
    'Maya',
    'Otto',
    'Vera'
  ]),
  Series.str('dept', [
    'eng',
    'eng',
    'design',
    'eng',
    'data',
    'data',
    'design',
    'eng',
    'data',
    'design'
  ]),
  Series.i64(
    'salary',
    [85000, 92000, 78000, 65000, 88000, 71000, 73000, 95000, 82000, 79000]
  ),
  Series.str('joined', [
    '2018-03-15',
    '2020-09-01',
    '2019-06-22',
    '2023-01-10',
    '2021-04-05',
    '2022-11-12',
    '2017-08-30',
    '2019-12-01',
    '2020-02-14',
    '2024-05-20'
  ])
])

const departments = new DataFrame([
  Series.str('code', ['eng', 'design', 'data']),
  Series.str('dept_name', ['Engineering', 'Design', 'Data Science']),
  Series.str('manager', ['Lena', 'Omar', 'Ines'])
])

section(`polar-bare + ${version()}`)
console.log(
  `${employees.height()} employees across ${departments.height()} departments`
)
employees.print()

// ── 1. categorize via when/then/otherwise ───────────────────────────────
section('1. Salary tiers (when/then/otherwise)')
employees
  .lazy()
  .withColumn(
    when(col('salary').gtEq(90000))
      .then(lit('senior'))
      .when(col('salary').gtEq(75000))
      .then(lit('mid'))
      .otherwise(lit('junior'))
      .alias('tier')
  )
  .select([col('name'), col('dept'), col('salary'), col('tier')])
  .collect()
  .sort(['dept', 'name'])
  .print()

// ── 2. groupBy + multi-expression agg ───────────────────────────────────
section('2. Headcount, mean & max salary per department')
employees
  .groupBy(['dept'])
  .agg([
    col('id').count().alias('headcount'),
    col('salary').mean().cast('i64').alias('mean_salary'),
    col('salary').max().alias('max_salary')
  ])
  .collect()
  .sort(['dept'])
  .print()

// ── 3. .str accessor: initials & name length ────────────────────────────
section('3. String ops: initials + name length')
employees
  .lazy()
  .select([
    col('name'),
    col('name').str.head(lit(1)).str.toUpperCase().alias('initial'),
    col('name').str.lenChars().alias('chars')
  ])
  .collect()
  .print()

// ── 4. .dt accessor: parse joined date, extract year / quarter / tenure ─
section('4. Date ops: parse joined date → year / quarter')
employees
  .lazy()
  .withColumn(col('joined').str.toDate('%Y-%m-%d').alias('joined_date'))
  .select([
    col('name'),
    col('joined_date').dt.year().alias('joined_year'),
    col('joined_date').dt.quarter().alias('joined_q'),
    col('joined_date').dt.monthEnd().dt.day().alias('mo_end_day')
  ])
  .collect()
  .sort(['joined_year', 'name'])
  .print()

// ── 5. join two frames ──────────────────────────────────────────────────
section('5. Join employees ⋈ departments on dept = code')
employees
  .lazy()
  .join(departments.lazy(), 'dept', 'code', 'inner')
  .select([col('name'), col('dept_name'), col('manager'), col('salary')])
  .collect()
  .sort(['dept_name', 'name'])
  .print()

// ── 6. window function via .over() ──────────────────────────────────────
section('6. Above-average earners within their department (over)')
employees
  .lazy()
  .withColumn(
    col('salary').mean().over(['dept']).cast('i64').alias('team_mean')
  )
  .filter(col('salary').gt(col('team_mean')))
  .select([col('name'), col('dept'), col('salary'), col('team_mean')])
  .collect()
  .sort(['dept', 'name'])
  .print()

// ── 7. IO roundtrip ─────────────────────────────────────────────────────
section('7. Write enriched report to /tmp, read back, compare')
const enriched = employees
  .lazy()
  .withColumn(
    col('joined').str.toDate('%Y-%m-%d').dt.year().alias('joined_year')
  )
  .collect()

const path = '/tmp/bare-polars-demo.csv'
enriched.writeCSV(path)
const reread = DataFrame.readCSV(path)
console.log(
  `Wrote ${enriched.height()} rows to ${path}; roundtrip matches: ${enriched.equals(reread)}`
)

// ── 8. error handling ───────────────────────────────────────────────────
section('8. Errors throw cleanly, the process keeps running')
try {
  employees.column('salray') // typo
} catch (e) {
  console.log('Caught:  ' + e.message)
}
try {
  DataFrame.readCSV('/no/such/file.csv')
} catch (e) {
  console.log('Caught:  ' + e.message)
}
console.log('… and we are still alive.')

console.log('')
