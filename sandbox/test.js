const { Series, DataFrame, LazyFrame, SortOptions, SortMultipleOptions, GroupBy } = require('..')

let df = DataFrame.readCSV('./test/data.csv')

df.print()
df = df.dropNulls(["height"])
df.print()

df.setColumnNames(["abe", "kat", "hund"])
df.print()

df.groupBy(["null"])
df.print()
