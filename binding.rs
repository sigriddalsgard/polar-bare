use bare_rust::{
    Array, Boolean, Env, Error as JsError, External, Function, Null, Number, Object, String,
    Undefined, Value, bare_exports,
};
use polars::prelude::*;
use polars::series::ops::NullBehavior;

fn js_err(env: &Env, e: impl std::fmt::Display) -> Value {
    JsError::new(env, &e.to_string()).into()
}

fn js_msg(env: &Env, msg: &str) -> Value {
    JsError::new(env, msg).into()
}

fn parse_dtype(s: &str) -> Option<DataType> {
    match s.to_lowercase().as_str() {
        "i8" | "int8" => Some(DataType::Int8),
        "i16" | "int16" => Some(DataType::Int16),
        "i32" | "int32" => Some(DataType::Int32),
        "i64" | "int64" => Some(DataType::Int64),
        "u8" | "uint8" => Some(DataType::UInt8),
        "u16" | "uint16" => Some(DataType::UInt16),
        "u32" | "uint32" => Some(DataType::UInt32),
        "u64" | "uint64" => Some(DataType::UInt64),
        "f32" | "float32" => Some(DataType::Float32),
        "f64" | "float64" => Some(DataType::Float64),
        "bool" | "boolean" => Some(DataType::Boolean),
        "str" | "string" => Some(DataType::String),
        _ => None,
    }
}

fn any_value_to_js(env: &Env, av: AnyValue<'_>) -> Value {
    match av {
        AnyValue::Null => Null::new(env).into(),
        AnyValue::Boolean(b) => Boolean::new(env, b).into(),
        AnyValue::Int8(v) => Number::with_i64(env, v as i64).into(),
        AnyValue::Int16(v) => Number::with_i64(env, v as i64).into(),
        AnyValue::Int32(v) => Number::with_i64(env, v as i64).into(),
        AnyValue::Int64(v) => Number::with_i64(env, v).into(),
        AnyValue::UInt8(v) => Number::with_i64(env, v as i64).into(),
        AnyValue::UInt16(v) => Number::with_i64(env, v as i64).into(),
        AnyValue::UInt32(v) => Number::with_i64(env, v as i64).into(),
        AnyValue::UInt64(v) => Number::with_f64(env, v as f64).into(),
        AnyValue::Float32(v) => Number::with_f64(env, v as f64).into(),
        AnyValue::Float64(v) => Number::with_f64(env, v).into(),
        AnyValue::String(s) => String::new(env, s).unwrap().into(),
        AnyValue::StringOwned(s) => String::new(env, s.as_str()).unwrap().into(),
        _ => Null::new(env).into(),
    }
}

fn read_bool(opts: &Object, name: &str) -> Option<bool> {
    if !opts.has_named_property(name).unwrap_or(false) {
        return None;
    }
    let b: Boolean = opts.get_named_property(name).ok()?;
    Some(b.into())
}

fn read_bool_array(opts: &Object, name: &str) -> Option<Vec<bool>> {
    if !opts.has_named_property(name).unwrap_or(false) {
        return None;
    }
    let arr: Array = opts.get_named_property(name).ok()?;
    Some(
        (0..arr.len())
            .map(|i| {
                let b: Boolean = arr.get(i).unwrap();
                b.into()
            })
            .collect(),
    )
}

fn build_sort_options(opts: &Object) -> SortOptions {
    let mut o = SortOptions::default();
    if let Some(v) = read_bool(opts, "descending") {
        o = o.with_order_descending(v);
    }
    if let Some(v) = read_bool(opts, "nullsLast") {
        o = o.with_nulls_last(v);
    }
    if let Some(v) = read_bool(opts, "multithreaded") {
        o = o.with_multithreaded(v);
    }
    if let Some(v) = read_bool(opts, "maintainOrder") {
        o = o.with_maintain_order(v);
    }
    o
}

fn build_sort_multiple_options(opts: &Object) -> SortMultipleOptions {
    let mut o = SortMultipleOptions::default();
    if let Some(v) = read_bool_array(opts, "descending") {
        o = o.with_order_descending_multi(v);
    }
    if let Some(v) = read_bool_array(opts, "nullsLast") {
        o = o.with_nulls_last_multi(v);
    }
    if let Some(v) = read_bool(opts, "multithreaded") {
        o = o.with_multithreaded(v);
    }
    if let Some(v) = read_bool(opts, "maintainOrder") {
        o = o.with_maintain_order(v);
    }
    o
}

bare_exports!(polar_bare_addon_exports, |env| {
    let mut exports = Object::new(&env)?;

    // ── IO ──────────────────────────────────────────────────────────────
    let read_csv = Function::new(&env, |env, info| {
        let path: String = info.arg(0).unwrap();
        let path_string: std::string::String = path.into();

        let reader = CsvReadOptions::default()
            .with_has_header(true)
            .try_into_reader_with_file_path(Some(path_string.clone().into()))
            .map_err(|e| js_err(env, e))?;
        let df = reader.finish().map_err(|e| js_err(env, e))?;
        println!("readCSV: {} rows from {}", df.height(), path_string);

        Ok(External::new(env, df)?.into())
    })?;

    let read_json = Function::new(&env, |env, info| {
        let path: String = info.arg(0).unwrap();
        let path_string: std::string::String = path.into();

        let mut file = std::fs::File::open(&path_string)
            .map_err(|e| js_err(env, format!("failed to open {path_string}: {e}")))?;
        let df = JsonReader::new(&mut file)
            .finish()
            .map_err(|e| js_err(env, e))?;
        println!("readJSON: {} rows from {}", df.height(), path_string);

        Ok(External::new(env, df)?.into())
    })?;

    // ── Printing ────────────────────────────────────────────────────────
    let data_frame_print = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        println!("{df}");
        Ok(Undefined::new(env).into())
    })?;

    let print_series = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        println!("{series}");
        Ok(Undefined::new(env).into())
    })?;

    // ── DataFrame construction & metadata ───────────────────────────────
    let data_frame_new = Function::new(&env, |env, info| {
        let columns: Array = info.arg(0).unwrap();

        let cols: Vec<Column> = (0..columns.len())
            .map(|i| {
                let val: External = columns.get(i).unwrap();
                let s: &Series = val.as_ref();
                s.clone().into()
            })
            .collect();

        let df = DataFrame::new(cols).map_err(|e| js_err(env, e))?;
        Ok(External::new(env, df)?.into())
    })?;

    let data_frame_clone = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        Ok(External::new(env, df.clone())?.into())
    })?;

    let data_frame_shape = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        let (rows, cols) = df.shape();
        println!("dfShape: ({rows}, {cols})");

        let mut array = Array::new(env, 2)?;
        array.set(0, Number::with_i64(env, rows as i64))?;
        array.set(1, Number::with_i64(env, cols as i64))?;
        Ok(array.into())
    })?;

    let data_frame_width = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        let result = df.width();
        println!("dfWidth: {result}");
        Ok(Number::with_i64(env, result as i64).into())
    })?;

    let data_frame_height = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        let result = df.height();
        println!("dfHeight: {result}");
        Ok(Number::with_i64(env, result as i64).into())
    })?;

    let data_frame_size = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        let result = df.size();
        println!("dfSize: {result}");
        Ok(Number::with_i64(env, result as i64).into())
    })?;

    let data_frame_is_empty = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        let result = df.is_empty();
        println!("isEmpty: {result}");
        Ok(Boolean::new(env, result).into())
    })?;

    let data_frame_column_names = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        let names = df.get_column_names_str();
        println!("dfColumns: {:?}", names);

        let mut res = Array::new(env, names.len())?;
        for i in 0..names.len() {
            let s = String::new(env, *names.get(i).unwrap())?;
            res.set(i as u32, s)?;
        }
        Ok(res.into())
    })?;

    let data_frame_set_column_names = Function::new(&env, |env, info| {
        let mut external: External = info.arg(0).unwrap();
        let names: Array = info.arg(1).unwrap();

        let names: Vec<std::string::String> = (0..names.len())
            .map(|i| {
                let val: String = names.get(i).unwrap();
                val.into()
            })
            .collect();

        let df: &mut DataFrame = external.as_mut();
        df.set_column_names(&names).map_err(|e| js_err(env, e))?;
        Ok(Undefined::new(env).into())
    })?;

    // ── DataFrame selection / slicing ───────────────────────────────────
    let data_frame_select = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let columns: Array = info.arg(1).unwrap();

        let cols: Vec<std::string::String> = (0..columns.len())
            .map(|i| {
                let val: String = columns.get(i).unwrap();
                val.into()
            })
            .collect();

        let df: &DataFrame = external.as_ref();
        let df = df.select(cols).map_err(|e| js_err(env, e))?;
        Ok(External::new(env, df)?.into())
    })?;

    let data_frame_column = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let name: String = info.arg(1).unwrap();
        let name: std::string::String = name.into();

        let df: &DataFrame = external.as_ref();
        let column = df.column(&name).map_err(|e| js_err(env, e))?;
        let series = column.as_materialized_series().clone();
        Ok(External::new(env, series)?.into())
    })?;

    let data_frame_head = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let length: Option<Number> = info.arg(1);
        let length_usize: Option<usize> = length.map(|n| {
            let v: i64 = n.into();
            v as usize
        });

        let df: &DataFrame = external.as_ref();
        let result = df.head(length_usize);
        Ok(External::new(env, result)?.into())
    })?;

    let data_frame_tail = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let length: Option<Number> = info.arg(1);
        let length_usize: Option<usize> = length.map(|n| {
            let v: i64 = n.into();
            v as usize
        });

        let df: &DataFrame = external.as_ref();
        let result = df.tail(length_usize);
        Ok(External::new(env, result)?.into())
    })?;

    let data_frame_slice = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let offset: Number = info.arg(1).unwrap();
        let length: Number = info.arg(2).unwrap();

        let offset: i64 = offset.into();
        let length_val: i64 = length.into();

        let df: &DataFrame = external.as_ref();
        let result = df.slice(offset, length_val as usize);
        Ok(External::new(env, result)?.into())
    })?;

    let data_frame_split_at = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let offset: Number = info.arg(1).unwrap();
        let offset: i64 = offset.into();

        let df: &DataFrame = external.as_ref();
        let (a, b) = df.split_at(offset);
        let mut arr = Array::new(env, 2)?;
        arr.set(0, External::new(env, a)?)?;
        arr.set(1, External::new(env, b)?)?;
        Ok(arr.into())
    })?;

    let data_frame_shift = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let periods: Number = info.arg(1).unwrap();
        let periods_val: i64 = periods.into();

        let df: &DataFrame = external.as_ref();
        let result = df.shift(periods_val);
        Ok(External::new(env, result)?.into())
    })?;

    let data_frame_reverse = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        let result = df.reverse();
        Ok(External::new(env, result)?.into())
    })?;

    // ── DataFrame mutation ──────────────────────────────────────────────
    let data_frame_drop = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let name: String = info.arg(1).unwrap();
        let name: std::string::String = name.into();

        let df: &DataFrame = external.as_ref();
        let result = df.drop(&name).map_err(|e| js_err(env, e))?;
        Ok(External::new(env, result)?.into())
    })?;

    let data_frame_pop = Function::new(&env, |env, info| {
        let mut external: External = info.arg(0).unwrap();
        let name: String = info.arg(1).unwrap();
        let name: std::string::String = name.into();

        let df: &mut DataFrame = external.as_mut();
        let column = df.drop_in_place(&name).map_err(|e| js_err(env, e))?;
        let series = column.as_materialized_series().clone();
        println!("popped: {series}");
        Ok(External::new(env, series)?.into())
    })?;

    let data_frame_drop_nulls = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let subset: Option<Array> = info.arg(1);

        let df: &DataFrame = external.as_ref();
        let result = match subset {
            Some(subset) => {
                let subset: Vec<std::string::String> = (0..subset.len())
                    .map(|i| {
                        let name: String = subset.get(i).unwrap();
                        name.into()
                    })
                    .collect();
                df.drop_nulls(Some(&subset[..]))
                    .map_err(|e| js_err(env, e))?
            }
            None => df
                .drop_nulls::<std::string::String>(None)
                .map_err(|e| js_err(env, e))?,
        };
        Ok(External::new(env, result)?.into())
    })?;

    let data_frame_rename = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let column: String = info.arg(1).unwrap();
        let name: String = info.arg(2).unwrap();
        let column: std::string::String = column.into();
        let name: std::string::String = name.into();

        let df: &DataFrame = external.as_ref();
        let mut df = df.clone();
        df.rename(&column, name.into())
            .map_err(|e| js_err(env, e))?;
        Ok(External::new(env, df)?.into())
    })?;

    let data_frame_replace = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let column: String = info.arg(1).unwrap();
        let new_col: External = info.arg(2).unwrap();

        let column: std::string::String = column.into();
        let new_col: &Series = new_col.as_ref();

        println!("{new_col}");

        let df: &DataFrame = external.as_ref();
        let mut df = df.clone();
        df.replace(&column, new_col.clone())
            .map_err(|e| js_err(env, e))?;
        Ok(External::new(env, df)?.into())
    })?;

    let data_frame_set_column = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let column: String = info.arg(1).unwrap();
        let new_col: External = info.arg(2).unwrap();

        let column: std::string::String = column.into();
        let new_col: &Series = new_col.as_ref();

        println!("{new_col}");

        let df: &DataFrame = external.as_ref();
        let mut df = df.clone();
        df.replace_or_add(column.into(), new_col.clone())
            .map_err(|e| js_err(env, e))?;
        Ok(External::new(env, df)?.into())
    })?;

    let data_frame_replace_column = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let index: Number = info.arg(1).unwrap();
        let new_col: External = info.arg(2).unwrap();

        let new_col: &Series = new_col.as_ref();
        let index: i64 = index.into();

        println!("{new_col}");

        let df: &DataFrame = external.as_ref();
        let mut df = df.clone();
        df.replace_column(index as usize, new_col.clone())
            .map_err(|e| js_err(env, e))?;
        Ok(External::new(env, df)?.into())
    })?;

    let data_frame_apply = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let name: String = info.arg(1).unwrap();
        let f: Function = info.arg(2).unwrap();
        let name: std::string::String = name.into();

        let df: &DataFrame = external.as_ref();
        let mut df_clone = df.clone();

        // Polars' apply signature is `FnOnce(&Column) -> C` with no error channel.
        // If the JS callback throws (or we hit a marshalling error), park the JS
        // error here, return the original column unchanged so polars can finish,
        // then re-throw on the way out.
        let error_slot: std::cell::Cell<Option<Value>> = std::cell::Cell::new(None);

        let apply_result = df_clone.apply(&name, |col| {
            let series: Series = col.as_materialized_series().clone();

            let arg: External = match External::new(env, series.clone()) {
                Ok(a) => a,
                Err(e) => {
                    error_slot.set(Some(e));
                    return series;
                }
            };
            let ctx = match Object::new(env) {
                Ok(c) => c,
                Err(e) => {
                    error_slot.set(Some(e));
                    return series;
                }
            };
            let args: Vec<Value> = vec![arg.into()];
            let call_result: Result<External, Value> = f.call(ctx, args.iter());
            match call_result {
                Ok(res) => {
                    let new_series: &Series = res.as_ref();
                    new_series.clone()
                }
                Err(e) => {
                    error_slot.set(Some(e));
                    series
                }
            }
        });

        if let Some(e) = error_slot.into_inner() {
            return Err(e);
        }
        apply_result.map_err(|e| js_err(env, e))?;

        Ok(External::new(env, df_clone)?.into())
    })?;

    // ── DataFrame combinations ──────────────────────────────────────────
    let data_frame_vstack = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let other: External = info.arg(1).unwrap();

        let df: &DataFrame = external.as_ref();
        let other: &DataFrame = other.as_ref();
        let df = df.vstack(other).map_err(|e| js_err(env, e))?;
        Ok(External::new(env, df)?.into())
    })?;

    let data_frame_vstack_mut = Function::new(&env, |env, info| {
        let mut external: External = info.arg(0).unwrap();
        let other: External = info.arg(1).unwrap();

        let df: &mut DataFrame = external.as_mut();
        let other: &DataFrame = other.as_ref();
        df.vstack_mut(other).map_err(|e| js_err(env, e))?;
        Ok(Undefined::new(env).into())
    })?;

    let data_frame_extend = Function::new(&env, |env, info| {
        let mut external: External = info.arg(0).unwrap();
        let other: External = info.arg(1).unwrap();

        let df: &mut DataFrame = external.as_mut();
        let other: &DataFrame = other.as_ref();
        df.extend(other).map_err(|e| js_err(env, e))?;
        Ok(Undefined::new(env).into())
    })?;

    // ── DataFrame analysis ──────────────────────────────────────────────
    let data_frame_null_count = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        let result = df.null_count();
        Ok(External::new(env, result)?.into())
    })?;

    let data_frame_is_unique = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        let chunked = df.is_unique().map_err(|e| js_err(env, e))?;
        Ok(External::new(env, chunked.into_series())?.into())
    })?;

    let data_frame_is_duplicated = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        let chunked = df.is_duplicated().map_err(|e| js_err(env, e))?;
        Ok(External::new(env, chunked.into_series())?.into())
    })?;

    let data_frame_min_horizontal = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        let column = df
            .min_horizontal()
            .map_err(|e| js_err(env, e))?
            .ok_or_else(|| js_msg(env, "min_horizontal: empty DataFrame"))?;
        let series = column.as_materialized_series().clone();
        Ok(External::new(env, series)?.into())
    })?;

    let data_frame_max_horizontal = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        let column = df
            .max_horizontal()
            .map_err(|e| js_err(env, e))?
            .ok_or_else(|| js_msg(env, "max_horizontal: empty DataFrame"))?;
        let series = column.as_materialized_series().clone();
        Ok(External::new(env, series)?.into())
    })?;

    let data_frame_sum_horizontal = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        let column = df
            .sum_horizontal(NullStrategy::Ignore)
            .map_err(|e| js_err(env, e))?
            .ok_or_else(|| js_msg(env, "sum_horizontal: empty DataFrame"))?;
        let series = column.as_materialized_series().clone();
        Ok(External::new(env, series)?.into())
    })?;

    let data_frame_mean_horizontal = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        let column = df
            .mean_horizontal(NullStrategy::Ignore)
            .map_err(|e| js_err(env, e))?
            .ok_or_else(|| js_msg(env, "mean_horizontal: empty DataFrame"))?;
        let series = column.as_materialized_series().clone();
        Ok(External::new(env, series)?.into())
    })?;

    let data_frame_filter = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let mask_ext: External = info.arg(1).unwrap();

        let df: &DataFrame = external.as_ref();
        let mask: &Series = mask_ext.as_ref();
        let mask_ca = mask.bool().map_err(|e| js_err(env, e))?;
        let result = df.filter(mask_ca).map_err(|e| js_err(env, e))?;
        Ok(External::new(env, result)?.into())
    })?;

    let data_frame_with_column = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series_ext: External = info.arg(1).unwrap();

        let df: &DataFrame = external.as_ref();
        let new_col: &Series = series_ext.as_ref();
        let mut df = df.clone();
        df.with_column(new_col.clone())
            .map_err(|e| js_err(env, e))?;
        Ok(External::new(env, df)?.into())
    })?;

    let data_frame_with_columns = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let columns: Array = info.arg(1).unwrap();

        let df: &DataFrame = external.as_ref();
        let mut df = df.clone();
        for i in 0..columns.len() {
            let ext: External = columns.get(i).unwrap();
            let s: &Series = ext.as_ref();
            df.with_column(s.clone()).map_err(|e| js_err(env, e))?;
        }
        Ok(External::new(env, df)?.into())
    })?;

    let data_frame_hstack = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let columns: Array = info.arg(1).unwrap();

        let cols: Vec<Column> = (0..columns.len())
            .map(|i| {
                let ext: External = columns.get(i).unwrap();
                let s: &Series = ext.as_ref();
                s.clone().into()
            })
            .collect();

        let df: &DataFrame = external.as_ref();
        let result = df.hstack(&cols).map_err(|e| js_err(env, e))?;
        Ok(External::new(env, result)?.into())
    })?;

    let data_frame_unique = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let subset: Option<Array> = info.arg(1);

        let df: &DataFrame = external.as_ref();
        let lf = df.clone().lazy();
        let result = match subset {
            Some(arr) => {
                let names: Vec<PlSmallStr> = (0..arr.len())
                    .map(|i| {
                        let v: String = arr.get(i).unwrap();
                        let s: std::string::String = v.into();
                        PlSmallStr::from_string(s)
                    })
                    .collect();
                lf.unique(Some(by_name(names, false)), UniqueKeepStrategy::First)
                    .collect()
                    .map_err(|e| js_err(env, e))?
            }
            None => lf
                .unique(None, UniqueKeepStrategy::First)
                .collect()
                .map_err(|e| js_err(env, e))?,
        };
        Ok(External::new(env, result)?.into())
    })?;

    let data_frame_sample = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let n: Number = info.arg(1).unwrap();
        let with_replacement: Option<Boolean> = info.arg(2);
        let shuffle: Option<Boolean> = info.arg(3);
        let seed: Option<Number> = info.arg(4);

        let n: i64 = n.into();
        let wr: bool = with_replacement.map(|b| b.into()).unwrap_or(false);
        let sh: bool = shuffle.map(|b| b.into()).unwrap_or(false);
        let sd: Option<u64> = seed.map(|s| {
            let v: i64 = s.into();
            v as u64
        });

        let df: &DataFrame = external.as_ref();
        let n_series = Series::new("n".into(), &[n]);
        let result = df
            .sample_n(&n_series, wr, sh, sd)
            .map_err(|e| js_err(env, e))?;
        Ok(External::new(env, result)?.into())
    })?;

    let data_frame_transpose = Function::new(&env, |env, info| {
        let mut external: External = info.arg(0).unwrap();
        let df: &mut DataFrame = external.as_mut();
        let result = df.transpose(None, None).map_err(|e| js_err(env, e))?;
        Ok(External::new(env, result)?.into())
    })?;

    let data_frame_sum = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        let result = df
            .clone()
            .lazy()
            .sum()
            .collect()
            .map_err(|e| js_err(env, e))?;
        Ok(External::new(env, result)?.into())
    })?;

    let data_frame_mean = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        let result = df
            .clone()
            .lazy()
            .mean()
            .collect()
            .map_err(|e| js_err(env, e))?;
        Ok(External::new(env, result)?.into())
    })?;

    let data_frame_min = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        let result = df
            .clone()
            .lazy()
            .min()
            .collect()
            .map_err(|e| js_err(env, e))?;
        Ok(External::new(env, result)?.into())
    })?;

    let data_frame_max = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        let result = df
            .clone()
            .lazy()
            .max()
            .collect()
            .map_err(|e| js_err(env, e))?;
        Ok(External::new(env, result)?.into())
    })?;

    let data_frame_median = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        let result = df
            .clone()
            .lazy()
            .median()
            .collect()
            .map_err(|e| js_err(env, e))?;
        Ok(External::new(env, result)?.into())
    })?;

    let data_frame_equals = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let other_ext: External = info.arg(1).unwrap();
        let a: &DataFrame = external.as_ref();
        let b: &DataFrame = other_ext.as_ref();
        let result = a.equals(b);
        println!("dfEquals: {result}");
        Ok(Boolean::new(env, result).into())
    })?;

    let data_frame_estimated_size = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        let bytes = df.estimated_size();
        println!("dfEstimatedSize: {bytes} bytes");
        Ok(Number::with_i64(env, bytes as i64).into())
    })?;

    let data_frame_write_csv = Function::new(&env, |env, info| {
        let mut external: External = info.arg(0).unwrap();
        let path: String = info.arg(1).unwrap();
        let path_string: std::string::String = path.into();

        let df: &mut DataFrame = external.as_mut();
        let mut file = std::fs::File::create(&path_string)
            .map_err(|e| js_err(env, format!("failed to open {path_string}: {e}")))?;
        let rows = df.height();
        CsvWriter::new(&mut file)
            .finish(df)
            .map_err(|e| js_err(env, e))?;
        println!("writeCSV: {rows} rows to {path_string}");
        Ok(Undefined::new(env).into())
    })?;

    let data_frame_write_json = Function::new(&env, |env, info| {
        let mut external: External = info.arg(0).unwrap();
        let path: String = info.arg(1).unwrap();
        let path_string: std::string::String = path.into();

        let df: &mut DataFrame = external.as_mut();
        let mut file = std::fs::File::create(&path_string)
            .map_err(|e| js_err(env, format!("failed to open {path_string}: {e}")))?;
        let rows = df.height();
        JsonWriter::new(&mut file)
            .with_json_format(JsonFormat::Json)
            .finish(df)
            .map_err(|e| js_err(env, e))?;
        println!("writeJSON: {rows} rows to {path_string}");
        Ok(Undefined::new(env).into())
    })?;

    // ── DataFrame explode ───────────────────────────────────────────────
    // Det er ikke muligt at eksplodere endnu, da en dataframe/series ikke
    // kan have en array inde i en kolonne
    let data_frame_explode = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let columns: Array = info.arg(1).unwrap();

        let cols: Vec<std::string::String> = (0..columns.len())
            .map(|i| {
                let val: String = columns.get(i).unwrap();
                val.into()
            })
            .collect();

        let df: &DataFrame = external.as_ref();
        let result = df.explode(cols).map_err(|e| js_err(env, e))?;
        Ok(External::new(env, result)?.into())
    })?;

    // ── DataFrame sort (options is a plain JS object) ───────────────────
    let data_frame_sort = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let columns: Array = info.arg(1).unwrap();
        let options: Object = info.arg(2).unwrap();

        let cols: Vec<std::string::String> = (0..columns.len())
            .map(|i| {
                let val: String = columns.get(i).unwrap();
                val.into()
            })
            .collect();

        let options = build_sort_multiple_options(&options);

        let df: &DataFrame = external.as_ref();
        let df = df.sort(cols, options).map_err(|e| js_err(env, e))?;
        Ok(External::new(env, df)?.into())
    })?;

    // ── DataFrame group_by → LazyGroupBy ────────────────────────────────
    let data_frame_group_by = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let by: Array = info.arg(1).unwrap();

        let by_exprs: Vec<Expr> = (0..by.len())
            .map(|i| {
                let val: String = by.get(i).unwrap();
                let s: std::string::String = val.into();
                col(s.as_str())
            })
            .collect();

        let df: &DataFrame = external.as_ref();
        let lgb = df.clone().lazy().group_by(by_exprs);
        Ok(External::new(env, lgb)?.into())
    })?;

    // ── DataFrame ↔ LazyFrame ───────────────────────────────────────────
    let data_frame_lazy = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let df: &DataFrame = external.as_ref();
        Ok(External::new(env, df.clone().lazy())?.into())
    })?;

    // ── Series construction ─────────────────────────────────────────────
    let series_new_i8 = Function::new(&env, |env, info| {
        let header: String = info.arg(0).unwrap();
        let column: Array = info.arg(1).unwrap();

        let cols: Vec<i8> = (0..column.len())
            .map(|i| {
                let val: Number = column.get(i).unwrap();
                let val: i64 = val.into();
                val as i8
            })
            .collect();

        let header: std::string::String = header.into();
        let series: Series = Series::new(header.into(), &cols);
        Ok(External::new(env, series)?.into())
    })?;

    let series_new_i64 = Function::new(&env, |env, info| {
        let header: String = info.arg(0).unwrap();
        let column: Array = info.arg(1).unwrap();

        let cols: Vec<i64> = (0..column.len())
            .map(|i| {
                let val: Number = column.get(i).unwrap();
                val.into()
            })
            .collect();

        let header: std::string::String = header.into();
        let series: Series = Series::new(header.into(), &cols);
        Ok(External::new(env, series)?.into())
    })?;

    let series_new_str = Function::new(&env, |env, info| {
        let header: String = info.arg(0).unwrap();
        let column: Array = info.arg(1).unwrap();

        let cols: Vec<std::string::String> = (0..column.len())
            .map(|i| {
                let val: String = column.get(i).unwrap();
                val.into()
            })
            .collect();

        let header: std::string::String = header.into();
        let series: Series = Series::new(header.into(), &cols);
        Ok(External::new(env, series)?.into())
    })?;

    // ── Series methods ──────────────────────────────────────────────────
    let series_clear = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        Ok(External::new(env, series.clear())?.into())
    })?;

    let series_to_data_frame = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        Ok(External::new(env, series.clone().into_frame())?.into())
    })?;

    let series_to_float = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        let result = series.to_float().map_err(|e| js_err(env, e))?;
        Ok(External::new(env, result)?.into())
    })?;

    let sort_series = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let options: Object = info.arg(1).unwrap();
        let options = build_sort_options(&options);

        let series: &Series = external.as_ref();
        let sorted = series.sort(options).map_err(|e| js_err(env, e))?;
        Ok(External::new(env, sorted)?.into())
    })?;

    let series_sum = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        let v: f64 = series.sum().map_err(|e| js_err(env, e))?;
        println!("seriesSum: {v}");
        Ok(Number::with_f64(env, v).into())
    })?;

    let series_mean = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        let v = series.mean().unwrap_or(f64::NAN);
        println!("seriesMean: {v}");
        Ok(Number::with_f64(env, v).into())
    })?;

    let series_min = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        let v: f64 = series
            .min()
            .map_err(|e| js_err(env, e))?
            .unwrap_or(f64::NAN);
        println!("seriesMin: {v}");
        Ok(Number::with_f64(env, v).into())
    })?;

    let series_max = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        let v: f64 = series
            .max()
            .map_err(|e| js_err(env, e))?
            .unwrap_or(f64::NAN);
        println!("seriesMax: {v}");
        Ok(Number::with_f64(env, v).into())
    })?;

    let series_median = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        let v = series.median().unwrap_or(f64::NAN);
        println!("seriesMedian: {v}");
        Ok(Number::with_f64(env, v).into())
    })?;

    let series_len = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        let n = series.len();
        println!("seriesLen: {n}");
        Ok(Number::with_i64(env, n as i64).into())
    })?;

    let series_name = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        let name = series.name().as_str().to_string();
        println!("seriesName: {name}");
        Ok(String::new(env, &name)?.into())
    })?;

    let series_rename = Function::new(&env, |env, info| {
        let mut external: External = info.arg(0).unwrap();
        let name: String = info.arg(1).unwrap();
        let name: std::string::String = name.into();

        let series: &mut Series = external.as_mut();
        series.rename(name.into());
        Ok(Undefined::new(env).into())
    })?;

    let series_dtype = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        let dt = format!("{}", series.dtype());
        println!("seriesDtype: {dt}");
        Ok(String::new(env, &dt)?.into())
    })?;

    let series_cast = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let dtype: String = info.arg(1).unwrap();
        let dtype: std::string::String = dtype.into();
        let dt = parse_dtype(&dtype)
            .ok_or_else(|| js_msg(env, &format!("unsupported dtype: {dtype}")))?;

        let series: &Series = external.as_ref();
        let casted = series.cast(&dt).map_err(|e| js_err(env, e))?;
        Ok(External::new(env, casted)?.into())
    })?;

    let series_null_count = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        let n = series.null_count();
        println!("seriesNullCount: {n}");
        Ok(Number::with_i64(env, n as i64).into())
    })?;

    let series_is_null = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        Ok(External::new(env, series.is_null().into_series())?.into())
    })?;

    let series_is_not_null = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        Ok(External::new(env, series.is_not_null().into_series())?.into())
    })?;

    let series_drop_nulls = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        Ok(External::new(env, series.drop_nulls())?.into())
    })?;

    let series_fill_null = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let strategy: String = info.arg(1).unwrap();
        let strategy: std::string::String = strategy.into();

        let s = match strategy.as_str() {
            "forward" => FillNullStrategy::Forward(None),
            "backward" => FillNullStrategy::Backward(None),
            "mean" => FillNullStrategy::Mean,
            "min" => FillNullStrategy::Min,
            "max" => FillNullStrategy::Max,
            "zero" => FillNullStrategy::Zero,
            "one" => FillNullStrategy::One,
            _ => {
                return Err(js_msg(
                    env,
                    &format!("unsupported fillNull strategy: {strategy}"),
                ));
            }
        };

        let series: &Series = external.as_ref();
        let filled = series.fill_null(s).map_err(|e| js_err(env, e))?;
        Ok(External::new(env, filled)?.into())
    })?;

    let series_unique = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        let u = series.unique().map_err(|e| js_err(env, e))?;
        Ok(External::new(env, u)?.into())
    })?;

    let series_n_unique = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        let n = series.n_unique().map_err(|e| js_err(env, e))?;
        println!("seriesNUnique: {n}");
        Ok(Number::with_i64(env, n as i64).into())
    })?;

    let series_head = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let length: Option<Number> = info.arg(1);
        let length_usize: Option<usize> = length.map(|n| {
            let v: i64 = n.into();
            v as usize
        });

        let series: &Series = external.as_ref();
        Ok(External::new(env, series.head(length_usize))?.into())
    })?;

    let series_tail = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let length: Option<Number> = info.arg(1);
        let length_usize: Option<usize> = length.map(|n| {
            let v: i64 = n.into();
            v as usize
        });

        let series: &Series = external.as_ref();
        Ok(External::new(env, series.tail(length_usize))?.into())
    })?;

    let series_slice = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let offset: Number = info.arg(1).unwrap();
        let length: Number = info.arg(2).unwrap();
        let offset: i64 = offset.into();
        let length: i64 = length.into();

        let series: &Series = external.as_ref();
        Ok(External::new(env, series.slice(offset, length as usize))?.into())
    })?;

    let series_reverse = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        Ok(External::new(env, series.reverse())?.into())
    })?;

    let series_shift = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let periods: Number = info.arg(1).unwrap();
        let periods: i64 = periods.into();

        let series: &Series = external.as_ref();
        Ok(External::new(env, series.shift(periods))?.into())
    })?;

    let series_arg_sort = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let opts: Object = info.arg(1).unwrap();
        let opts = build_sort_options(&opts);

        let series: &Series = external.as_ref();
        let idx = series.arg_sort(opts);
        Ok(External::new(env, idx.into_series())?.into())
    })?;

    let series_arg_min = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        let result = series.arg_min();
        println!("seriesArgMin: {:?}", result);
        match result {
            Some(v) => Ok(Number::with_i64(env, v as i64).into()),
            None => Ok(Null::new(env).into()),
        }
    })?;

    let series_arg_max = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        let result = series.arg_max();
        println!("seriesArgMax: {:?}", result);
        match result {
            Some(v) => Ok(Number::with_i64(env, v as i64).into()),
            None => Ok(Null::new(env).into()),
        }
    })?;

    let series_std = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        let scalar = series.std_reduce(1).map_err(|e| js_err(env, e))?;
        let v: f64 = scalar.value().extract().unwrap_or(f64::NAN);
        println!("seriesStd: {v}");
        Ok(Number::with_f64(env, v).into())
    })?;

    let series_var = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        let scalar = series.var_reduce(1).map_err(|e| js_err(env, e))?;
        let v: f64 = scalar.value().extract().unwrap_or(f64::NAN);
        println!("seriesVar: {v}");
        Ok(Number::with_f64(env, v).into())
    })?;

    let series_quantile = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let q: Number = info.arg(1).unwrap();
        let q: f64 = q.into();

        let series: &Series = external.as_ref();
        let scalar = series
            .quantile_reduce(q, QuantileMethod::Nearest)
            .map_err(|e| js_err(env, e))?;
        let v: f64 = scalar.value().extract().unwrap_or(f64::NAN);
        println!("seriesQuantile({q}): {v}");
        Ok(Number::with_f64(env, v).into())
    })?;

    let series_equals = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let other_ext: External = info.arg(1).unwrap();
        let a: &Series = external.as_ref();
        let b: &Series = other_ext.as_ref();
        let result = a.equals(b);
        println!("seriesEquals: {result}");
        Ok(Boolean::new(env, result).into())
    })?;

    let series_get = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let idx: Number = info.arg(1).unwrap();
        let idx: i64 = idx.into();

        let series: &Series = external.as_ref();
        let av = series.get(idx as usize).map_err(|e| js_err(env, e))?;
        Ok(any_value_to_js(env, av))
    })?;

    let series_to_array = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        let len = series.len();
        let mut arr = Array::new(env, len)?;
        for i in 0..len {
            let av = series.get(i).unwrap();
            arr.set(i as u32, any_value_to_js(env, av))?;
        }
        Ok(arr.into())
    })?;

    let series_value_counts = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series: &Series = external.as_ref();
        let df = series
            .value_counts(false, false, "count".into(), false)
            .map_err(|e| js_err(env, e))?;
        Ok(External::new(env, df)?.into())
    })?;

    let series_new_f64 = Function::new(&env, |env, info| {
        let header: String = info.arg(0).unwrap();
        let column: Array = info.arg(1).unwrap();

        let cols: Vec<f64> = (0..column.len())
            .map(|i| {
                let val: Number = column.get(i).unwrap();
                val.into()
            })
            .collect();

        let header: std::string::String = header.into();
        let series: Series = Series::new(header.into(), &cols);
        Ok(External::new(env, series)?.into())
    })?;

    let series_new_bool = Function::new(&env, |env, info| {
        let header: String = info.arg(0).unwrap();
        let column: Array = info.arg(1).unwrap();

        let cols: Vec<bool> = (0..column.len())
            .map(|i| {
                let val: Boolean = column.get(i).unwrap();
                val.into()
            })
            .collect();

        let header: std::string::String = header.into();
        let series: Series = Series::new(header.into(), &cols);
        Ok(External::new(env, series)?.into())
    })?;

    // ── GroupBy aggregations (operate on LazyGroupBy → LazyFrame) ───────
    let group_by_sum = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lgb: &LazyGroupBy = external.as_ref();
        let lf = lgb.clone().agg([col("*").sum()]);
        Ok(External::new(env, lf)?.into())
    })?;

    let group_by_mean = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lgb: &LazyGroupBy = external.as_ref();
        let lf = lgb.clone().agg([col("*").mean()]);
        Ok(External::new(env, lf)?.into())
    })?;

    let group_by_min = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lgb: &LazyGroupBy = external.as_ref();
        let lf = lgb.clone().agg([col("*").min()]);
        Ok(External::new(env, lf)?.into())
    })?;

    let group_by_max = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lgb: &LazyGroupBy = external.as_ref();
        let lf = lgb.clone().agg([col("*").max()]);
        Ok(External::new(env, lf)?.into())
    })?;

    let group_by_median = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lgb: &LazyGroupBy = external.as_ref();
        let lf = lgb.clone().agg([col("*").median()]);
        Ok(External::new(env, lf)?.into())
    })?;

    let group_by_count = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lgb: &LazyGroupBy = external.as_ref();
        let lf = lgb.clone().agg([col("*").count()]);
        Ok(External::new(env, lf)?.into())
    })?;

    let group_by_first = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lgb: &LazyGroupBy = external.as_ref();
        let lf = lgb.clone().agg([col("*").first()]);
        Ok(External::new(env, lf)?.into())
    })?;

    let group_by_last = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lgb: &LazyGroupBy = external.as_ref();
        let lf = lgb.clone().agg([col("*").last()]);
        Ok(External::new(env, lf)?.into())
    })?;

    // ── LazyFrame ───────────────────────────────────────────────────────
    let lazy_frame_collect = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lf: &LazyFrame = external.as_ref();
        let df = lf.clone().collect().map_err(|e| js_err(env, e))?;
        Ok(External::new(env, df)?.into())
    })?;

    let lazy_frame_count = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().count())?.into())
    })?;

    let lazy_frame_max = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().max())?.into())
    })?;

    let lazy_frame_mean = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().mean())?.into())
    })?;

    let lazy_frame_median = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().median())?.into())
    })?;

    let lazy_frame_min = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().min())?.into())
    })?;

    let lazy_frame_sum = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().sum())?.into())
    })?;

    let lazy_frame_first = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().first())?.into())
    })?;

    let lazy_frame_last = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().last())?.into())
    })?;

    let lazy_frame_select = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let columns: Array = info.arg(1).unwrap();

        let exprs: Vec<Expr> = (0..columns.len())
            .map(|i| {
                let val: String = columns.get(i).unwrap();
                let s: std::string::String = val.into();
                col(s.as_str())
            })
            .collect();

        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().select(exprs))?.into())
    })?;

    let lazy_frame_filter = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let col_name: String = info.arg(1).unwrap();
        let op: String = info.arg(2).unwrap();
        let value: Number = info.arg(3).unwrap();

        let col_name: std::string::String = col_name.into();
        let op: std::string::String = op.into();
        let value: f64 = value.into();

        let c = col(col_name.as_str());
        let expr = match op.as_str() {
            ">" => c.gt(lit(value)),
            ">=" => c.gt_eq(lit(value)),
            "<" => c.lt(lit(value)),
            "<=" => c.lt_eq(lit(value)),
            "==" => c.eq(lit(value)),
            "!=" => c.neq(lit(value)),
            _ => return Err(js_msg(env, &format!("unsupported filter op: {op}"))),
        };

        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().filter(expr))?.into())
    })?;

    let lazy_frame_group_by = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let by: Array = info.arg(1).unwrap();

        let by_exprs: Vec<Expr> = (0..by.len())
            .map(|i| {
                let val: String = by.get(i).unwrap();
                let s: std::string::String = val.into();
                col(s.as_str())
            })
            .collect();

        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().group_by(by_exprs))?.into())
    })?;

    let lazy_frame_join = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let other: External = info.arg(1).unwrap();
        let left_on: String = info.arg(2).unwrap();
        let right_on: String = info.arg(3).unwrap();
        let how: String = info.arg(4).unwrap();

        let left_on: std::string::String = left_on.into();
        let right_on: std::string::String = right_on.into();
        let how: std::string::String = how.into();

        let lf: &LazyFrame = external.as_ref();
        let other: &LazyFrame = other.as_ref();
        let l = col(left_on.as_str());
        let r = col(right_on.as_str());
        let joined = match how.as_str() {
            "inner" => lf.clone().inner_join(other.clone(), l, r),
            "left" => lf.clone().left_join(other.clone(), l, r),
            "full" => lf.clone().full_join(other.clone(), l, r),
            _ => return Err(js_msg(env, &format!("unsupported join type: {how}"))),
        };
        Ok(External::new(env, joined)?.into())
    })?;

    let lazy_frame_with_column = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let series_ext: External = info.arg(1).unwrap();

        let lf: &LazyFrame = external.as_ref();
        let series: &Series = series_ext.as_ref();
        Ok(External::new(env, lf.clone().with_column(lit(series.clone())))?.into())
    })?;

    let lazy_frame_with_columns = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let columns: Array = info.arg(1).unwrap();

        let exprs: Vec<Expr> = (0..columns.len())
            .map(|i| {
                let ext: External = columns.get(i).unwrap();
                let s: &Series = ext.as_ref();
                lit(s.clone())
            })
            .collect();

        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().with_columns(exprs))?.into())
    })?;

    let lazy_frame_sort = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let columns: Array = info.arg(1).unwrap();
        let options: Object = info.arg(2).unwrap();

        let cols: Vec<PlSmallStr> = (0..columns.len())
            .map(|i| {
                let val: String = columns.get(i).unwrap();
                let s: std::string::String = val.into();
                PlSmallStr::from_string(s)
            })
            .collect();

        let options = build_sort_multiple_options(&options);
        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(
            env,
            lf.clone().sort_by_exprs(
                cols.iter().map(|c| col(c.as_str())).collect::<Vec<_>>(),
                options,
            ),
        )?
        .into())
    })?;

    let lazy_frame_reverse = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().reverse())?.into())
    })?;

    let lazy_frame_head = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let n: Number = info.arg(1).unwrap();
        let n: i64 = n.into();
        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().slice(0, n as u32))?.into())
    })?;

    let lazy_frame_tail = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let n: Number = info.arg(1).unwrap();
        let n: i64 = n.into();
        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().tail(n as u32))?.into())
    })?;

    let lazy_frame_slice = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let offset: Number = info.arg(1).unwrap();
        let length: Number = info.arg(2).unwrap();
        let offset: i64 = offset.into();
        let length: i64 = length.into();

        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().slice(offset, length as u32))?.into())
    })?;

    let lazy_frame_unique = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let subset: Option<Array> = info.arg(1);

        let lf: &LazyFrame = external.as_ref();
        let result = match subset {
            Some(arr) => {
                let names: Vec<PlSmallStr> = (0..arr.len())
                    .map(|i| {
                        let v: String = arr.get(i).unwrap();
                        let s: std::string::String = v.into();
                        PlSmallStr::from_string(s)
                    })
                    .collect();
                lf.clone()
                    .unique(Some(by_name(names, false)), UniqueKeepStrategy::First)
            }
            None => lf.clone().unique(None, UniqueKeepStrategy::First),
        };
        Ok(External::new(env, result)?.into())
    })?;

    let lazy_frame_drop_nulls = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let subset: Option<Array> = info.arg(1);

        let lf: &LazyFrame = external.as_ref();
        let result = match subset {
            Some(arr) => {
                let names: Vec<PlSmallStr> = (0..arr.len())
                    .map(|i| {
                        let v: String = arr.get(i).unwrap();
                        let s: std::string::String = v.into();
                        PlSmallStr::from_string(s)
                    })
                    .collect();
                lf.clone().drop_nulls(Some(by_name(names, false)))
            }
            None => lf.clone().drop_nulls(None),
        };
        Ok(External::new(env, result)?.into())
    })?;

    let lazy_frame_rename = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let from: String = info.arg(1).unwrap();
        let to: String = info.arg(2).unwrap();
        let from: std::string::String = from.into();
        let to: std::string::String = to.into();

        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().rename([from], [to], true))?.into())
    })?;

    let lazy_frame_drop = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let columns: Array = info.arg(1).unwrap();

        let names: Vec<PlSmallStr> = (0..columns.len())
            .map(|i| {
                let v: String = columns.get(i).unwrap();
                let s: std::string::String = v.into();
                PlSmallStr::from_string(s)
            })
            .collect();

        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().drop(by_name(names, false)))?.into())
    })?;

    let lazy_frame_explode = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let columns: Array = info.arg(1).unwrap();

        let names: Vec<PlSmallStr> = (0..columns.len())
            .map(|i| {
                let v: String = columns.get(i).unwrap();
                let s: std::string::String = v.into();
                PlSmallStr::from_string(s)
            })
            .collect();

        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().explode(by_name(names, false)))?.into())
    })?;

    let lazy_frame_clone = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone())?.into())
    })?;

    let lazy_frame_explain = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let optimized: Option<Boolean> = info.arg(1);
        let opt: bool = optimized.map(|b| b.into()).unwrap_or(true);

        let lf: &LazyFrame = external.as_ref();
        let plan = lf.clone().explain(opt).map_err(|e| js_err(env, e))?;
        Ok(String::new(env, &plan)?.into())
    })?;

    let lazy_frame_std = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().std(1))?.into())
    })?;

    let lazy_frame_var = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().var(1))?.into())
    })?;

    let lazy_frame_quantile = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let q: Number = info.arg(1).unwrap();
        let q: f64 = q.into();

        let lf: &LazyFrame = external.as_ref();
        let lf_q = lf.clone().quantile(lit(q), QuantileMethod::Nearest);
        Ok(External::new(env, lf_q)?.into())
    })?;

    // LazyGroupBy extensions
    let group_by_std = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lgb: &LazyGroupBy = external.as_ref();
        let lf = lgb.clone().agg([col("*").std(1)]);
        Ok(External::new(env, lf)?.into())
    })?;

    let group_by_var = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let lgb: &LazyGroupBy = external.as_ref();
        let lf = lgb.clone().agg([col("*").var(1)]);
        Ok(External::new(env, lf)?.into())
    })?;

    let group_by_quantile = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let q: Number = info.arg(1).unwrap();
        let q: f64 = q.into();

        let lgb: &LazyGroupBy = external.as_ref();
        let lf = lgb
            .clone()
            .agg([col("*").quantile(lit(q), QuantileMethod::Nearest)]);
        Ok(External::new(env, lf)?.into())
    })?;

    let group_by_head = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let n: Number = info.arg(1).unwrap();
        let n: i64 = n.into();

        let lgb: &LazyGroupBy = external.as_ref();
        let lf = lgb.clone().head(Some(n as usize));
        Ok(External::new(env, lf)?.into())
    })?;

    let group_by_tail = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let n: Number = info.arg(1).unwrap();
        let n: i64 = n.into();

        let lgb: &LazyGroupBy = external.as_ref();
        let lf = lgb.clone().tail(Some(n as usize));
        Ok(External::new(env, lf)?.into())
    })?;

    // ── Expr DSL ────────────────────────────────────────────────────────
    let expr_col = Function::new(&env, |env, info| {
        let name: String = info.arg(0).unwrap();
        let name: std::string::String = name.into();
        Ok(External::new(env, col(name.as_str()))?.into())
    })?;

    let expr_cols = Function::new(&env, |env, info| {
        let names: Array = info.arg(0).unwrap();
        let names: Vec<PlSmallStr> = (0..names.len())
            .map(|i| {
                let v: String = names.get(i).unwrap();
                let s: std::string::String = v.into();
                PlSmallStr::from_string(s)
            })
            .collect();
        Ok(External::new(env, cols(names))?.into())
    })?;

    let expr_all = Function::new(&env, |env, _info| Ok(External::new(env, col("*"))?.into()))?;

    let expr_lit_f64 = Function::new(&env, |env, info| {
        let v: Number = info.arg(0).unwrap();
        let v: f64 = v.into();
        Ok(External::new(env, lit(v))?.into())
    })?;

    let expr_lit_i64 = Function::new(&env, |env, info| {
        let v: Number = info.arg(0).unwrap();
        let v: i64 = v.into();
        Ok(External::new(env, lit(v))?.into())
    })?;

    let expr_lit_str = Function::new(&env, |env, info| {
        let v: String = info.arg(0).unwrap();
        let v: std::string::String = v.into();
        Ok(External::new(env, lit(v))?.into())
    })?;

    let expr_lit_bool = Function::new(&env, |env, info| {
        let v: Boolean = info.arg(0).unwrap();
        let v: bool = v.into();
        Ok(External::new(env, lit(v))?.into())
    })?;

    let expr_lit_series = Function::new(&env, |env, info| {
        let series_ext: External = info.arg(0).unwrap();
        let series: &Series = series_ext.as_ref();
        Ok(External::new(env, lit(series.clone()))?.into())
    })?;

    // ── Expr arithmetic ─────────────────────────────────────────────────
    let expr_add = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let b: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let b: &Expr = b.as_ref();
        Ok(External::new(env, a.clone() + b.clone())?.into())
    })?;

    let expr_sub = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let b: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let b: &Expr = b.as_ref();
        Ok(External::new(env, a.clone() - b.clone())?.into())
    })?;

    let expr_mul = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let b: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let b: &Expr = b.as_ref();
        Ok(External::new(env, a.clone() * b.clone())?.into())
    })?;

    let expr_div = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let b: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let b: &Expr = b.as_ref();
        Ok(External::new(env, a.clone() / b.clone())?.into())
    })?;

    let expr_mod = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let b: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let b: &Expr = b.as_ref();
        Ok(External::new(env, a.clone() % b.clone())?.into())
    })?;

    let expr_neg = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, lit(0i64) - a.clone())?.into())
    })?;

    // ── Expr comparison ─────────────────────────────────────────────────
    let expr_eq = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let b: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let b: &Expr = b.as_ref();
        Ok(External::new(env, a.clone().eq(b.clone()))?.into())
    })?;

    let expr_neq = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let b: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let b: &Expr = b.as_ref();
        Ok(External::new(env, a.clone().neq(b.clone()))?.into())
    })?;

    let expr_lt = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let b: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let b: &Expr = b.as_ref();
        Ok(External::new(env, a.clone().lt(b.clone()))?.into())
    })?;

    let expr_lt_eq = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let b: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let b: &Expr = b.as_ref();
        Ok(External::new(env, a.clone().lt_eq(b.clone()))?.into())
    })?;

    let expr_gt = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let b: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let b: &Expr = b.as_ref();
        Ok(External::new(env, a.clone().gt(b.clone()))?.into())
    })?;

    let expr_gt_eq = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let b: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let b: &Expr = b.as_ref();
        Ok(External::new(env, a.clone().gt_eq(b.clone()))?.into())
    })?;

    // ── Expr logical ────────────────────────────────────────────────────
    let expr_and = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let b: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let b: &Expr = b.as_ref();
        Ok(External::new(env, a.clone().and(b.clone()))?.into())
    })?;

    let expr_or = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let b: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let b: &Expr = b.as_ref();
        Ok(External::new(env, a.clone().or(b.clone()))?.into())
    })?;

    let expr_xor = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let b: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let b: &Expr = b.as_ref();
        Ok(External::new(env, a.clone().xor(b.clone()))?.into())
    })?;

    let expr_not = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().not())?.into())
    })?;

    // ── Expr transforms ─────────────────────────────────────────────────
    let expr_alias = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let name: String = info.arg(1).unwrap();
        let name: std::string::String = name.into();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().alias(name.as_str()))?.into())
    })?;

    let expr_cast = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let dtype: String = info.arg(1).unwrap();
        let dtype: std::string::String = dtype.into();
        let dt = parse_dtype(&dtype).expect("unsupported dtype");
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().cast(dt))?.into())
    })?;

    let expr_is_null = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().is_null())?.into())
    })?;

    let expr_is_not_null = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().is_not_null())?.into())
    })?;

    let expr_fill_null = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let fill: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let f: &Expr = fill.as_ref();
        Ok(External::new(env, a.clone().fill_null(f.clone()))?.into())
    })?;

    // ── Expr aggregations ───────────────────────────────────────────────
    let expr_sum = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().sum())?.into())
    })?;

    let expr_mean = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().mean())?.into())
    })?;

    let expr_min = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().min())?.into())
    })?;

    let expr_max = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().max())?.into())
    })?;

    let expr_median = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().median())?.into())
    })?;

    let expr_std = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().std(1))?.into())
    })?;

    let expr_var = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().var(1))?.into())
    })?;

    let expr_count = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().count())?.into())
    })?;

    let expr_n_unique = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().n_unique())?.into())
    })?;

    let expr_first = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().first())?.into())
    })?;

    let expr_last = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().last())?.into())
    })?;

    // ── Expr math / numerical ───────────────────────────────────────────
    let expr_abs = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().abs())?.into())
    })?;

    let expr_sign = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().sign())?.into())
    })?;

    let expr_sqrt = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().sqrt())?.into())
    })?;

    let expr_cbrt = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().cbrt())?.into())
    })?;

    let expr_pow = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let exp_ext: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let e: &Expr = exp_ext.as_ref();
        Ok(External::new(env, a.clone().pow(e.clone()))?.into())
    })?;

    let expr_log = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let base_ext: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let b: &Expr = base_ext.as_ref();
        Ok(External::new(env, a.clone().log(b.clone()))?.into())
    })?;

    let expr_log1p = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().log1p())?.into())
    })?;

    let expr_exp = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().exp())?.into())
    })?;

    let expr_floor = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().floor())?.into())
    })?;

    let expr_ceil = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().ceil())?.into())
    })?;

    let expr_round = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let decimals: Number = info.arg(1).unwrap();
        let decimals: i64 = decimals.into();
        let a: &Expr = a.as_ref();
        Ok(External::new(
            env,
            a.clone()
                .round(decimals as u32, RoundMode::HalfAwayFromZero),
        )?
        .into())
    })?;

    let expr_clip = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let min_ext: External = info.arg(1).unwrap();
        let max_ext: External = info.arg(2).unwrap();
        let a: &Expr = a.as_ref();
        let mn: &Expr = min_ext.as_ref();
        let mx: &Expr = max_ext.as_ref();
        Ok(External::new(env, a.clone().clip(mn.clone(), mx.clone()))?.into())
    })?;

    let expr_clip_min = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let min_ext: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let mn: &Expr = min_ext.as_ref();
        Ok(External::new(env, a.clone().clip_min(mn.clone()))?.into())
    })?;

    let expr_clip_max = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let max_ext: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let mx: &Expr = max_ext.as_ref();
        Ok(External::new(env, a.clone().clip_max(mx.clone()))?.into())
    })?;

    let expr_sin = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().sin())?.into())
    })?;

    let expr_cos = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().cos())?.into())
    })?;

    let expr_tan = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().tan())?.into())
    })?;

    let expr_asin = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().arcsin())?.into())
    })?;

    let expr_acos = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().arccos())?.into())
    })?;

    let expr_atan = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().arctan())?.into())
    })?;

    let expr_sinh = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().sinh())?.into())
    })?;

    let expr_cosh = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().cosh())?.into())
    })?;

    let expr_tanh = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().tanh())?.into())
    })?;

    let expr_quantile = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let q: Number = info.arg(1).unwrap();
        let q: f64 = q.into();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().quantile(lit(q), QuantileMethod::Nearest))?.into())
    })?;

    // ── Expr ordering / selection ───────────────────────────────────────
    let expr_shift = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let n_ext: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let n: &Expr = n_ext.as_ref();
        Ok(External::new(env, a.clone().shift(n.clone()))?.into())
    })?;

    let expr_diff = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let n_ext: External = info.arg(1).unwrap();
        let nb: String = info.arg(2).unwrap();
        let nb: std::string::String = nb.into();
        let null_behavior = match nb.as_str() {
            "drop" => NullBehavior::Drop,
            "ignore" => NullBehavior::Ignore,
            _ => return Err(js_msg(env, &format!("unsupported nullBehavior: {nb}"))),
        };
        let a: &Expr = a.as_ref();
        let n: &Expr = n_ext.as_ref();
        Ok(External::new(env, a.clone().diff(n.clone(), null_behavior))?.into())
    })?;

    let expr_is_in = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let other_ext: External = info.arg(1).unwrap();
        let nulls_equal: Boolean = info.arg(2).unwrap();
        let a: &Expr = a.as_ref();
        let other: &Expr = other_ext.as_ref();
        Ok(External::new(env, a.clone().is_in(other.clone(), nulls_equal.into()))?.into())
    })?;

    let expr_is_between = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let low_ext: External = info.arg(1).unwrap();
        let high_ext: External = info.arg(2).unwrap();
        let closed: String = info.arg(3).unwrap();
        let closed: std::string::String = closed.into();
        let closed_interval = match closed.as_str() {
            "both" => ClosedInterval::Both,
            "left" => ClosedInterval::Left,
            "right" => ClosedInterval::Right,
            "none" => ClosedInterval::None,
            _ => {
                return Err(js_msg(
                    env,
                    &format!("unsupported closed interval: {closed}"),
                ));
            }
        };
        let a: &Expr = a.as_ref();
        let low: &Expr = low_ext.as_ref();
        let high: &Expr = high_ext.as_ref();
        Ok(External::new(
            env,
            a.clone()
                .is_between(low.clone(), high.clone(), closed_interval),
        )?
        .into())
    })?;

    let expr_rank = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let opts: Object = info.arg(1).unwrap();
        let method = if opts.has_named_property("method").unwrap_or(false) {
            let m: String = opts.get_named_property("method").unwrap();
            let m: std::string::String = m.into();
            match m.as_str() {
                "average" => RankMethod::Average,
                "min" => RankMethod::Min,
                "max" => RankMethod::Max,
                "dense" => RankMethod::Dense,
                "ordinal" => RankMethod::Ordinal,
                _ => return Err(js_msg(env, &format!("unsupported rank method: {m}"))),
            }
        } else {
            RankMethod::Dense
        };
        let descending = read_bool(&opts, "descending").unwrap_or(false);
        let options = RankOptions { method, descending };

        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().rank(options, None))?.into())
    })?;

    let expr_top_k = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let k_ext: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let k: &Expr = k_ext.as_ref();
        Ok(External::new(env, a.clone().top_k(k.clone()))?.into())
    })?;

    let expr_bottom_k = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let k_ext: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let k: &Expr = k_ext.as_ref();
        Ok(External::new(env, a.clone().bottom_k(k.clone()))?.into())
    })?;

    // ── when/then/otherwise (single-branch for now) ─────────────────────
    let expr_when_then_otherwise = Function::new(&env, |env, info| {
        let cond_ext: External = info.arg(0).unwrap();
        let then_ext: External = info.arg(1).unwrap();
        let else_ext: External = info.arg(2).unwrap();
        let cond: &Expr = cond_ext.as_ref();
        let then_val: &Expr = then_ext.as_ref();
        let else_val: &Expr = else_ext.as_ref();
        let result = when(cond.clone())
            .then(then_val.clone())
            .otherwise(else_val.clone());
        Ok(External::new(env, result)?.into())
    })?;

    let expr_over = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let by: Array = info.arg(1).unwrap();

        let by_exprs: Vec<Expr> = (0..by.len())
            .map(|i| {
                let v: String = by.get(i).unwrap();
                let s: std::string::String = v.into();
                col(s.as_str())
            })
            .collect();

        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().over(by_exprs))?.into())
    })?;

    // ── Expr.str ────────────────────────────────────────────────────────
    let expr_str_contains = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let pat: External = info.arg(1).unwrap();
        let strict: Boolean = info.arg(2).unwrap();
        let a: &Expr = a.as_ref();
        let p: &Expr = pat.as_ref();
        Ok(External::new(env, a.clone().str().contains(p.clone(), strict.into()))?.into())
    })?;

    let expr_str_contains_literal = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let pat: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let p: &Expr = pat.as_ref();
        Ok(External::new(env, a.clone().str().contains_literal(p.clone()))?.into())
    })?;

    let expr_str_starts_with = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let pat: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let p: &Expr = pat.as_ref();
        Ok(External::new(env, a.clone().str().starts_with(p.clone()))?.into())
    })?;

    let expr_str_ends_with = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let pat: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let p: &Expr = pat.as_ref();
        Ok(External::new(env, a.clone().str().ends_with(p.clone()))?.into())
    })?;

    let expr_str_to_lowercase = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().str().to_lowercase())?.into())
    })?;

    let expr_str_to_uppercase = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().str().to_uppercase())?.into())
    })?;

    let expr_str_len_chars = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().str().len_chars())?.into())
    })?;

    let expr_str_len_bytes = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().str().len_bytes())?.into())
    })?;

    let expr_str_replace = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let pat: External = info.arg(1).unwrap();
        let val: External = info.arg(2).unwrap();
        let literal: Boolean = info.arg(3).unwrap();
        let a: &Expr = a.as_ref();
        let p: &Expr = pat.as_ref();
        let v: &Expr = val.as_ref();
        Ok(External::new(
            env,
            a.clone()
                .str()
                .replace(p.clone(), v.clone(), literal.into()),
        )?
        .into())
    })?;

    let expr_str_replace_all = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let pat: External = info.arg(1).unwrap();
        let val: External = info.arg(2).unwrap();
        let literal: Boolean = info.arg(3).unwrap();
        let a: &Expr = a.as_ref();
        let p: &Expr = pat.as_ref();
        let v: &Expr = val.as_ref();
        Ok(External::new(
            env,
            a.clone()
                .str()
                .replace_all(p.clone(), v.clone(), literal.into()),
        )?
        .into())
    })?;

    let expr_str_slice = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let offset: External = info.arg(1).unwrap();
        let length: External = info.arg(2).unwrap();
        let a: &Expr = a.as_ref();
        let o: &Expr = offset.as_ref();
        let l: &Expr = length.as_ref();
        Ok(External::new(env, a.clone().str().slice(o.clone(), l.clone()))?.into())
    })?;

    let expr_str_head = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let n: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let nn: &Expr = n.as_ref();
        Ok(External::new(env, a.clone().str().head(nn.clone()))?.into())
    })?;

    let expr_str_tail = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let n: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let nn: &Expr = n.as_ref();
        Ok(External::new(env, a.clone().str().tail(nn.clone()))?.into())
    })?;

    let expr_str_strip_chars = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let matches: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let m: &Expr = matches.as_ref();
        Ok(External::new(env, a.clone().str().strip_chars(m.clone()))?.into())
    })?;

    let expr_str_strip_chars_start = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let matches: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let m: &Expr = matches.as_ref();
        Ok(External::new(env, a.clone().str().strip_chars_start(m.clone()))?.into())
    })?;

    let expr_str_strip_chars_end = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let matches: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let m: &Expr = matches.as_ref();
        Ok(External::new(env, a.clone().str().strip_chars_end(m.clone()))?.into())
    })?;

    let expr_str_split = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let by: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let b: &Expr = by.as_ref();
        Ok(External::new(env, a.clone().str().split(b.clone()))?.into())
    })?;

    let expr_str_pad_start = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let length: External = info.arg(1).unwrap();
        let fill: String = info.arg(2).unwrap();
        let fill: std::string::String = fill.into();
        let fill_char: char = fill.chars().next().unwrap_or(' ');
        let a: &Expr = a.as_ref();
        let l: &Expr = length.as_ref();
        Ok(External::new(env, a.clone().str().pad_start(l.clone(), fill_char))?.into())
    })?;

    let expr_str_pad_end = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let length: External = info.arg(1).unwrap();
        let fill: String = info.arg(2).unwrap();
        let fill: std::string::String = fill.into();
        let fill_char: char = fill.chars().next().unwrap_or(' ');
        let a: &Expr = a.as_ref();
        let l: &Expr = length.as_ref();
        Ok(External::new(env, a.clone().str().pad_end(l.clone(), fill_char))?.into())
    })?;

    let expr_str_zfill = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let length: External = info.arg(1).unwrap();
        let a: &Expr = a.as_ref();
        let l: &Expr = length.as_ref();
        Ok(External::new(env, a.clone().str().zfill(l.clone()))?.into())
    })?;

    let expr_str_find = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let pat: External = info.arg(1).unwrap();
        let strict: Boolean = info.arg(2).unwrap();
        let a: &Expr = a.as_ref();
        let p: &Expr = pat.as_ref();
        Ok(External::new(env, a.clone().str().find(p.clone(), strict.into()))?.into())
    })?;

    let expr_str_count_matches = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let pat: External = info.arg(1).unwrap();
        let literal: Boolean = info.arg(2).unwrap();
        let a: &Expr = a.as_ref();
        let p: &Expr = pat.as_ref();
        Ok(External::new(
            env,
            a.clone().str().count_matches(p.clone(), literal.into()),
        )?
        .into())
    })?;

    let expr_str_reverse = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().str().reverse())?.into())
    })?;

    let expr_str_to_date = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let format: Option<String> = info.arg(1);
        let format_opt: Option<PlSmallStr> = format.map(|f| {
            let s: std::string::String = f.into();
            PlSmallStr::from_string(s)
        });
        let options = StrptimeOptions {
            format: format_opt,
            strict: true,
            exact: true,
            cache: true,
        };
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().str().to_date(options))?.into())
    })?;

    // ── Expr.dt ─────────────────────────────────────────────────────────
    let expr_dt_year = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().dt().year())?.into())
    })?;

    let expr_dt_iso_year = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().dt().iso_year())?.into())
    })?;

    let expr_dt_quarter = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().dt().quarter())?.into())
    })?;

    let expr_dt_month = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().dt().month())?.into())
    })?;

    let expr_dt_week = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().dt().week())?.into())
    })?;

    let expr_dt_weekday = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().dt().weekday())?.into())
    })?;

    let expr_dt_day = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().dt().day())?.into())
    })?;

    let expr_dt_ordinal_day = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().dt().ordinal_day())?.into())
    })?;

    let expr_dt_hour = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().dt().hour())?.into())
    })?;

    let expr_dt_minute = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().dt().minute())?.into())
    })?;

    let expr_dt_second = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().dt().second())?.into())
    })?;

    let expr_dt_days_in_month = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().dt().days_in_month())?.into())
    })?;

    let expr_dt_month_start = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().dt().month_start())?.into())
    })?;

    let expr_dt_month_end = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().dt().month_end())?.into())
    })?;

    let expr_dt_strftime = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let format: String = info.arg(1).unwrap();
        let format: std::string::String = format.into();
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().dt().strftime(format.as_str()))?.into())
    })?;

    let expr_dt_timestamp = Function::new(&env, |env, info| {
        let a: External = info.arg(0).unwrap();
        let tu: String = info.arg(1).unwrap();
        let tu_str: std::string::String = tu.into();
        let time_unit = match tu_str.as_str() {
            "ns" | "nanoseconds" => TimeUnit::Nanoseconds,
            "us" | "microseconds" => TimeUnit::Microseconds,
            "ms" | "milliseconds" => TimeUnit::Milliseconds,
            _ => return Err(js_msg(env, &format!("unsupported time unit: {tu_str}"))),
        };
        let a: &Expr = a.as_ref();
        Ok(External::new(env, a.clone().dt().timestamp(time_unit))?.into())
    })?;

    // ── Lazy/Group consumers that take Expr ─────────────────────────────
    let lazy_frame_filter_expr = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let expr_ext: External = info.arg(1).unwrap();
        let lf: &LazyFrame = external.as_ref();
        let e: &Expr = expr_ext.as_ref();
        Ok(External::new(env, lf.clone().filter(e.clone()))?.into())
    })?;

    let lazy_frame_select_expr = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let exprs_arr: Array = info.arg(1).unwrap();
        let exprs: Vec<Expr> = (0..exprs_arr.len())
            .map(|i| {
                let e: External = exprs_arr.get(i).unwrap();
                let r: &Expr = e.as_ref();
                r.clone()
            })
            .collect();
        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().select(exprs))?.into())
    })?;

    let lazy_frame_with_column_expr = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let expr_ext: External = info.arg(1).unwrap();
        let lf: &LazyFrame = external.as_ref();
        let e: &Expr = expr_ext.as_ref();
        Ok(External::new(env, lf.clone().with_column(e.clone()))?.into())
    })?;

    let lazy_frame_with_columns_expr = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let exprs_arr: Array = info.arg(1).unwrap();
        let exprs: Vec<Expr> = (0..exprs_arr.len())
            .map(|i| {
                let e: External = exprs_arr.get(i).unwrap();
                let r: &Expr = e.as_ref();
                r.clone()
            })
            .collect();
        let lf: &LazyFrame = external.as_ref();
        Ok(External::new(env, lf.clone().with_columns(exprs))?.into())
    })?;

    let group_by_agg = Function::new(&env, |env, info| {
        let external: External = info.arg(0).unwrap();
        let exprs_arr: Array = info.arg(1).unwrap();
        let exprs: Vec<Expr> = (0..exprs_arr.len())
            .map(|i| {
                let e: External = exprs_arr.get(i).unwrap();
                let r: &Expr = e.as_ref();
                r.clone()
            })
            .collect();
        let lgb: &LazyGroupBy = external.as_ref();
        Ok(External::new(env, lgb.clone().agg(exprs))?.into())
    })?;

    let concat_data_frames = Function::new(&env, |env, info| {
        let frames: Array = info.arg(0).unwrap();

        let lfs: Vec<LazyFrame> = (0..frames.len())
            .map(|i| {
                let ext: External = frames.get(i).unwrap();
                let df: &DataFrame = ext.as_ref();
                df.clone().lazy()
            })
            .collect();

        let result = concat(lfs, UnionArgs::default())
            .map_err(|e| js_err(env, e))?
            .collect()
            .map_err(|e| js_err(env, e))?;
        Ok(External::new(env, result)?.into())
    })?;

    let polars_version = Function::new(&env, |env, _info| {
        Ok(String::new(env, "polars 0.51.0")?.into())
    })?;

    // ── Exports ─────────────────────────────────────────────────────────
    exports.set_named_property("readCSV", read_csv)?;
    exports.set_named_property("readJSON", read_json)?;

    exports.set_named_property("dataFramePrint", data_frame_print)?;
    exports.set_named_property("printSeries", print_series)?;

    exports.set_named_property("dataFrameNew", data_frame_new)?;
    exports.set_named_property("dataFrameClone", data_frame_clone)?;
    exports.set_named_property("dataFrameShape", data_frame_shape)?;
    exports.set_named_property("dataFrameWidth", data_frame_width)?;
    exports.set_named_property("dataFrameHeight", data_frame_height)?;
    exports.set_named_property("dataFrameSize", data_frame_size)?;
    exports.set_named_property("dataFrameIsEmpty", data_frame_is_empty)?;
    exports.set_named_property("dataFrameColumnNames", data_frame_column_names)?;
    exports.set_named_property("dataFrameSetColumnNames", data_frame_set_column_names)?;

    exports.set_named_property("dataFrameSelect", data_frame_select)?;
    exports.set_named_property("dataFrameColumn", data_frame_column)?;
    exports.set_named_property("dataFrameHead", data_frame_head)?;
    exports.set_named_property("dataFrameTail", data_frame_tail)?;
    exports.set_named_property("dataFrameSlice", data_frame_slice)?;
    exports.set_named_property("dataFrameSplitAt", data_frame_split_at)?;
    exports.set_named_property("dataFrameShift", data_frame_shift)?;
    exports.set_named_property("dataFrameReverse", data_frame_reverse)?;

    exports.set_named_property("dataFrameDrop", data_frame_drop)?;
    exports.set_named_property("dataFramePop", data_frame_pop)?;
    exports.set_named_property("dataFrameDropNulls", data_frame_drop_nulls)?;
    exports.set_named_property("dataFrameRename", data_frame_rename)?;
    exports.set_named_property("dataFrameReplace", data_frame_replace)?;
    exports.set_named_property("dataFrameSetColumn", data_frame_set_column)?;
    exports.set_named_property("dataFrameReplaceColumn", data_frame_replace_column)?;
    exports.set_named_property("dataFrameApply", data_frame_apply)?;

    exports.set_named_property("dataFrameVstack", data_frame_vstack)?;
    exports.set_named_property("dataFrameVstackMut", data_frame_vstack_mut)?;
    exports.set_named_property("dataFrameExtend", data_frame_extend)?;

    exports.set_named_property("dataFrameNullCount", data_frame_null_count)?;
    exports.set_named_property("dataFrameIsUnique", data_frame_is_unique)?;
    exports.set_named_property("dataFrameIsDuplicated", data_frame_is_duplicated)?;
    exports.set_named_property("dataFrameMinHorizontal", data_frame_min_horizontal)?;
    exports.set_named_property("dataFrameMaxHorizontal", data_frame_max_horizontal)?;

    exports.set_named_property("dataFrameExplode", data_frame_explode)?;
    exports.set_named_property("dataFrameSort", data_frame_sort)?;
    exports.set_named_property("dataFrameGroupBy", data_frame_group_by)?;
    exports.set_named_property("dataFrameLazy", data_frame_lazy)?;

    exports.set_named_property("SeriesI8", series_new_i8)?;
    exports.set_named_property("SeriesI64", series_new_i64)?;
    exports.set_named_property("SeriesStr", series_new_str)?;
    exports.set_named_property("seriesClear", series_clear)?;
    exports.set_named_property("seriesToDataFrame", series_to_data_frame)?;
    exports.set_named_property("seriesToFloat", series_to_float)?;
    exports.set_named_property("sortSeries", sort_series)?;
    exports.set_named_property("seriesSum", series_sum)?;
    exports.set_named_property("seriesMean", series_mean)?;
    exports.set_named_property("seriesMin", series_min)?;
    exports.set_named_property("seriesMax", series_max)?;
    exports.set_named_property("seriesMedian", series_median)?;
    exports.set_named_property("seriesLen", series_len)?;

    exports.set_named_property("groupBySum", group_by_sum)?;
    exports.set_named_property("groupByMean", group_by_mean)?;
    exports.set_named_property("groupByMin", group_by_min)?;
    exports.set_named_property("groupByMax", group_by_max)?;
    exports.set_named_property("groupByMedian", group_by_median)?;
    exports.set_named_property("groupByCount", group_by_count)?;
    exports.set_named_property("groupByFirst", group_by_first)?;
    exports.set_named_property("groupByLast", group_by_last)?;

    exports.set_named_property("lazyFrameCollect", lazy_frame_collect)?;
    exports.set_named_property("lazyFrameCount", lazy_frame_count)?;
    exports.set_named_property("lazyFrameMax", lazy_frame_max)?;
    exports.set_named_property("lazyFrameMean", lazy_frame_mean)?;
    exports.set_named_property("lazyFrameMedian", lazy_frame_median)?;
    exports.set_named_property("lazyFrameMin", lazy_frame_min)?;
    exports.set_named_property("lazyFrameSum", lazy_frame_sum)?;
    exports.set_named_property("lazyFrameFirst", lazy_frame_first)?;
    exports.set_named_property("lazyFrameLast", lazy_frame_last)?;
    exports.set_named_property("lazyFrameSelect", lazy_frame_select)?;
    exports.set_named_property("lazyFrameFilter", lazy_frame_filter)?;
    exports.set_named_property("lazyFrameGroupBy", lazy_frame_group_by)?;
    exports.set_named_property("lazyFrameJoin", lazy_frame_join)?;
    exports.set_named_property("lazyFrameWithColumn", lazy_frame_with_column)?;
    exports.set_named_property("lazyFrameWithColumns", lazy_frame_with_columns)?;
    exports.set_named_property("lazyFrameSort", lazy_frame_sort)?;
    exports.set_named_property("lazyFrameReverse", lazy_frame_reverse)?;
    exports.set_named_property("lazyFrameHead", lazy_frame_head)?;
    exports.set_named_property("lazyFrameTail", lazy_frame_tail)?;
    exports.set_named_property("lazyFrameSlice", lazy_frame_slice)?;
    exports.set_named_property("lazyFrameUnique", lazy_frame_unique)?;
    exports.set_named_property("lazyFrameDropNulls", lazy_frame_drop_nulls)?;
    exports.set_named_property("lazyFrameRename", lazy_frame_rename)?;
    exports.set_named_property("lazyFrameDrop", lazy_frame_drop)?;
    exports.set_named_property("lazyFrameExplode", lazy_frame_explode)?;
    exports.set_named_property("lazyFrameClone", lazy_frame_clone)?;
    exports.set_named_property("lazyFrameExplain", lazy_frame_explain)?;
    exports.set_named_property("lazyFrameStd", lazy_frame_std)?;
    exports.set_named_property("lazyFrameVar", lazy_frame_var)?;
    exports.set_named_property("lazyFrameQuantile", lazy_frame_quantile)?;

    exports.set_named_property("groupByStd", group_by_std)?;
    exports.set_named_property("groupByVar", group_by_var)?;
    exports.set_named_property("groupByQuantile", group_by_quantile)?;
    exports.set_named_property("groupByHead", group_by_head)?;
    exports.set_named_property("groupByTail", group_by_tail)?;

    // Series extensions
    exports.set_named_property("SeriesF64", series_new_f64)?;
    exports.set_named_property("SeriesBool", series_new_bool)?;
    exports.set_named_property("seriesName", series_name)?;
    exports.set_named_property("seriesRename", series_rename)?;
    exports.set_named_property("seriesDtype", series_dtype)?;
    exports.set_named_property("seriesCast", series_cast)?;
    exports.set_named_property("seriesNullCount", series_null_count)?;
    exports.set_named_property("seriesIsNull", series_is_null)?;
    exports.set_named_property("seriesIsNotNull", series_is_not_null)?;
    exports.set_named_property("seriesDropNulls", series_drop_nulls)?;
    exports.set_named_property("seriesFillNull", series_fill_null)?;
    exports.set_named_property("seriesUnique", series_unique)?;
    exports.set_named_property("seriesNUnique", series_n_unique)?;
    exports.set_named_property("seriesHead", series_head)?;
    exports.set_named_property("seriesTail", series_tail)?;
    exports.set_named_property("seriesSlice", series_slice)?;
    exports.set_named_property("seriesReverse", series_reverse)?;
    exports.set_named_property("seriesShift", series_shift)?;
    exports.set_named_property("seriesArgSort", series_arg_sort)?;
    exports.set_named_property("seriesArgMin", series_arg_min)?;
    exports.set_named_property("seriesArgMax", series_arg_max)?;
    exports.set_named_property("seriesStd", series_std)?;
    exports.set_named_property("seriesVar", series_var)?;
    exports.set_named_property("seriesQuantile", series_quantile)?;
    exports.set_named_property("seriesEquals", series_equals)?;
    exports.set_named_property("seriesGet", series_get)?;
    exports.set_named_property("seriesToArray", series_to_array)?;
    exports.set_named_property("seriesValueCounts", series_value_counts)?;

    // DataFrame extensions
    exports.set_named_property("dataFrameSumHorizontal", data_frame_sum_horizontal)?;
    exports.set_named_property("dataFrameMeanHorizontal", data_frame_mean_horizontal)?;
    exports.set_named_property("dataFrameFilter", data_frame_filter)?;
    exports.set_named_property("dataFrameWithColumn", data_frame_with_column)?;
    exports.set_named_property("dataFrameWithColumns", data_frame_with_columns)?;
    exports.set_named_property("dataFrameHstack", data_frame_hstack)?;
    exports.set_named_property("dataFrameUnique", data_frame_unique)?;
    exports.set_named_property("dataFrameSample", data_frame_sample)?;
    exports.set_named_property("dataFrameTranspose", data_frame_transpose)?;
    exports.set_named_property("dataFrameSum", data_frame_sum)?;
    exports.set_named_property("dataFrameMean", data_frame_mean)?;
    exports.set_named_property("dataFrameMin", data_frame_min)?;
    exports.set_named_property("dataFrameMax", data_frame_max)?;
    exports.set_named_property("dataFrameMedian", data_frame_median)?;
    exports.set_named_property("dataFrameEquals", data_frame_equals)?;
    exports.set_named_property("dataFrameEstimatedSize", data_frame_estimated_size)?;
    exports.set_named_property("dataFrameWriteCSV", data_frame_write_csv)?;
    exports.set_named_property("dataFrameWriteJSON", data_frame_write_json)?;

    exports.set_named_property("concatDataFrames", concat_data_frames)?;
    exports.set_named_property("polarsVersion", polars_version)?;

    // Expr DSL
    exports.set_named_property("exprCol", expr_col)?;
    exports.set_named_property("exprCols", expr_cols)?;
    exports.set_named_property("exprAll", expr_all)?;
    exports.set_named_property("exprLitF64", expr_lit_f64)?;
    exports.set_named_property("exprLitI64", expr_lit_i64)?;
    exports.set_named_property("exprLitStr", expr_lit_str)?;
    exports.set_named_property("exprLitBool", expr_lit_bool)?;
    exports.set_named_property("exprLitSeries", expr_lit_series)?;
    exports.set_named_property("exprAdd", expr_add)?;
    exports.set_named_property("exprSub", expr_sub)?;
    exports.set_named_property("exprMul", expr_mul)?;
    exports.set_named_property("exprDiv", expr_div)?;
    exports.set_named_property("exprMod", expr_mod)?;
    exports.set_named_property("exprNeg", expr_neg)?;
    exports.set_named_property("exprEq", expr_eq)?;
    exports.set_named_property("exprNeq", expr_neq)?;
    exports.set_named_property("exprLt", expr_lt)?;
    exports.set_named_property("exprLtEq", expr_lt_eq)?;
    exports.set_named_property("exprGt", expr_gt)?;
    exports.set_named_property("exprGtEq", expr_gt_eq)?;
    exports.set_named_property("exprAnd", expr_and)?;
    exports.set_named_property("exprOr", expr_or)?;
    exports.set_named_property("exprXor", expr_xor)?;
    exports.set_named_property("exprNot", expr_not)?;
    exports.set_named_property("exprAlias", expr_alias)?;
    exports.set_named_property("exprCast", expr_cast)?;
    exports.set_named_property("exprIsNull", expr_is_null)?;
    exports.set_named_property("exprIsNotNull", expr_is_not_null)?;
    exports.set_named_property("exprFillNull", expr_fill_null)?;
    exports.set_named_property("exprSum", expr_sum)?;
    exports.set_named_property("exprMean", expr_mean)?;
    exports.set_named_property("exprMin", expr_min)?;
    exports.set_named_property("exprMax", expr_max)?;
    exports.set_named_property("exprMedian", expr_median)?;
    exports.set_named_property("exprStd", expr_std)?;
    exports.set_named_property("exprVar", expr_var)?;
    exports.set_named_property("exprCount", expr_count)?;
    exports.set_named_property("exprNUnique", expr_n_unique)?;
    exports.set_named_property("exprFirst", expr_first)?;
    exports.set_named_property("exprLast", expr_last)?;
    exports.set_named_property("exprQuantile", expr_quantile)?;
    exports.set_named_property("exprAbs", expr_abs)?;
    exports.set_named_property("exprSign", expr_sign)?;
    exports.set_named_property("exprSqrt", expr_sqrt)?;
    exports.set_named_property("exprCbrt", expr_cbrt)?;
    exports.set_named_property("exprPow", expr_pow)?;
    exports.set_named_property("exprLog", expr_log)?;
    exports.set_named_property("exprLog1p", expr_log1p)?;
    exports.set_named_property("exprExp", expr_exp)?;
    exports.set_named_property("exprFloor", expr_floor)?;
    exports.set_named_property("exprCeil", expr_ceil)?;
    exports.set_named_property("exprRound", expr_round)?;
    exports.set_named_property("exprClip", expr_clip)?;
    exports.set_named_property("exprClipMin", expr_clip_min)?;
    exports.set_named_property("exprClipMax", expr_clip_max)?;
    exports.set_named_property("exprSin", expr_sin)?;
    exports.set_named_property("exprCos", expr_cos)?;
    exports.set_named_property("exprTan", expr_tan)?;
    exports.set_named_property("exprAsin", expr_asin)?;
    exports.set_named_property("exprAcos", expr_acos)?;
    exports.set_named_property("exprAtan", expr_atan)?;
    exports.set_named_property("exprSinh", expr_sinh)?;
    exports.set_named_property("exprCosh", expr_cosh)?;
    exports.set_named_property("exprTanh", expr_tanh)?;
    exports.set_named_property("exprShift", expr_shift)?;
    exports.set_named_property("exprDiff", expr_diff)?;
    exports.set_named_property("exprIsIn", expr_is_in)?;
    exports.set_named_property("exprIsBetween", expr_is_between)?;
    exports.set_named_property("exprRank", expr_rank)?;
    exports.set_named_property("exprTopK", expr_top_k)?;
    exports.set_named_property("exprBottomK", expr_bottom_k)?;
    exports.set_named_property("exprWhenThenOtherwise", expr_when_then_otherwise)?;
    exports.set_named_property("exprOver", expr_over)?;

    exports.set_named_property("lazyFrameFilterExpr", lazy_frame_filter_expr)?;
    exports.set_named_property("lazyFrameSelectExpr", lazy_frame_select_expr)?;
    exports.set_named_property("lazyFrameWithColumnExpr", lazy_frame_with_column_expr)?;
    exports.set_named_property("lazyFrameWithColumnsExpr", lazy_frame_with_columns_expr)?;
    exports.set_named_property("groupByAgg", group_by_agg)?;

    // Expr.str
    exports.set_named_property("exprStrContains", expr_str_contains)?;
    exports.set_named_property("exprStrContainsLiteral", expr_str_contains_literal)?;
    exports.set_named_property("exprStrStartsWith", expr_str_starts_with)?;
    exports.set_named_property("exprStrEndsWith", expr_str_ends_with)?;
    exports.set_named_property("exprStrToLowercase", expr_str_to_lowercase)?;
    exports.set_named_property("exprStrToUppercase", expr_str_to_uppercase)?;
    exports.set_named_property("exprStrLenChars", expr_str_len_chars)?;
    exports.set_named_property("exprStrLenBytes", expr_str_len_bytes)?;
    exports.set_named_property("exprStrReplace", expr_str_replace)?;
    exports.set_named_property("exprStrReplaceAll", expr_str_replace_all)?;
    exports.set_named_property("exprStrSlice", expr_str_slice)?;
    exports.set_named_property("exprStrHead", expr_str_head)?;
    exports.set_named_property("exprStrTail", expr_str_tail)?;
    exports.set_named_property("exprStrStripChars", expr_str_strip_chars)?;
    exports.set_named_property("exprStrStripCharsStart", expr_str_strip_chars_start)?;
    exports.set_named_property("exprStrStripCharsEnd", expr_str_strip_chars_end)?;
    exports.set_named_property("exprStrSplit", expr_str_split)?;
    exports.set_named_property("exprStrPadStart", expr_str_pad_start)?;
    exports.set_named_property("exprStrPadEnd", expr_str_pad_end)?;
    exports.set_named_property("exprStrZfill", expr_str_zfill)?;
    exports.set_named_property("exprStrFind", expr_str_find)?;
    exports.set_named_property("exprStrCountMatches", expr_str_count_matches)?;
    exports.set_named_property("exprStrReverse", expr_str_reverse)?;
    exports.set_named_property("exprStrToDate", expr_str_to_date)?;

    // Expr.dt
    exports.set_named_property("exprDtYear", expr_dt_year)?;
    exports.set_named_property("exprDtIsoYear", expr_dt_iso_year)?;
    exports.set_named_property("exprDtQuarter", expr_dt_quarter)?;
    exports.set_named_property("exprDtMonth", expr_dt_month)?;
    exports.set_named_property("exprDtWeek", expr_dt_week)?;
    exports.set_named_property("exprDtWeekday", expr_dt_weekday)?;
    exports.set_named_property("exprDtDay", expr_dt_day)?;
    exports.set_named_property("exprDtOrdinalDay", expr_dt_ordinal_day)?;
    exports.set_named_property("exprDtHour", expr_dt_hour)?;
    exports.set_named_property("exprDtMinute", expr_dt_minute)?;
    exports.set_named_property("exprDtSecond", expr_dt_second)?;
    exports.set_named_property("exprDtDaysInMonth", expr_dt_days_in_month)?;
    exports.set_named_property("exprDtMonthStart", expr_dt_month_start)?;
    exports.set_named_property("exprDtMonthEnd", expr_dt_month_end)?;
    exports.set_named_property("exprDtStrftime", expr_dt_strftime)?;
    exports.set_named_property("exprDtTimestamp", expr_dt_timestamp)?;

    Ok(exports.into())
});
