use std::{
    ops::{Index, RangeInclusive},
    str::FromStr,
};

use async_ui_web::{
    components::{Div, Input, Span, Table, Td, Th, Tr},
    join,
    prelude_traits::*,
    race, select, NoChild, ReactiveCell,
};

const NUM_COLUMNS: usize = 26;
const NUM_ROWS: usize = 100;

type Value = ReactiveCell<f64>;

struct State {
    values: Vec<Value>,
}
impl State {
    fn new() -> Self {
        Self {
            values: vec![ReactiveCell::new(f64::NAN); NUM_COLUMNS * NUM_ROWS],
        }
    }
}
#[derive(Clone, Copy, Debug)]
struct CellIndex(usize, usize);

impl FromStr for CellIndex {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let first = chars.next().ok_or(())?;
        if !first.is_ascii_uppercase() {
            return Err(());
        }
        let col = (first as usize) - ('A' as usize);
        let row = chars.as_str().parse().map_err(|_| ())?;
        Ok(Self(col, row))
    }
}

impl<T> Index<CellIndex> for Vec<T> {
    type Output = T;

    fn index(&self, index: CellIndex) -> &Self::Output {
        &self[index.0 * NUM_ROWS + index.1]
    }
}
pub async fn cells() {
    let state = State::new();
    let wrapper = Div::new();
    wrapper.add_class(style::wrapper);
    let table = Table::new();
    table.add_class(style::table);
    wrapper
        .render(
            table.render(join((
                Tr::new().render(join({
                    let mut headers = vec![Th::new().render("Cells".render())];
                    headers.extend((0..NUM_COLUMNS).map(|col| {
                        Th::new().render(
                            ((col as u8 + b'A') as char)
                                .encode_utf8(&mut [0; 1])
                                .render(),
                        )
                    }));
                    headers
                })),
                join(
                    (0..NUM_ROWS)
                        .map(|row| {
                            let tr = Tr::new();
                            tr.render(join((
                                Td::new().render(format!("{row}").render()),
                                join(
                                    (0..NUM_COLUMNS)
                                        .map(|col| cell(&state, CellIndex(col, row)))
                                        .collect::<Vec<_>>(),
                                ),
                            )))
                        })
                        .collect::<Vec<_>>(),
                ),
            ))),
        )
        .await;
}

async fn cell(state: &State, index: CellIndex) {
    let td = Td::new();
    td.add_class(style::cell);
    let input = Input::new();
    input.set_type("text");
    let computed = Span::new();
    let mut formula = Formula::Unknown;
    join((
        td.render(join((input.render(), computed.render(NoChild)))),
        async {
            loop {
                let v = formula.compute(state);
                computed.set_inner_text(&if v.is_finite() {
                    format!("={v:.3}")
                } else {
                    String::new()
                });
                *state.values[index].borrow_mut() = v;
                select!(
                    _ = input.until_change() => {
                        let text = input.value();
                        formula = Formula::from(&*text);
                    }
                    _ = formula.until_dep_change(state) => {}
                );
            }
        },
    ))
    .await;
}
#[derive(Debug)]
enum Formula {
    Literal(f64),
    Sum(RangeInclusive<CellIndex>),
    Unknown,
}
impl Formula {
    async fn until_dep_change(&self, state: &State) {
        match self {
            Formula::Sum(range) => {
                let (&CellIndex(fcol, frow), &CellIndex(tcol, trow)) = (range.start(), range.end());
                let mut cells = Vec::with_capacity((tcol - fcol + 1) * (trow - frow + 1));
                for col in fcol..=tcol {
                    for row in frow..=trow {
                        cells.push(state.values[CellIndex(col, row)].until_change())
                    }
                }
                race(cells).await;
            }
            _ => core::future::pending().await,
        }
    }
    fn compute(&self, state: &State) -> f64 {
        match self {
            Formula::Sum(range) => {
                let (&CellIndex(fcol, frow), &CellIndex(tcol, trow)) = (range.start(), range.end());
                let mut out = 0.0;
                for col in fcol..=tcol {
                    for row in frow..=trow {
                        out += *state.values[CellIndex(col, row)].borrow();
                    }
                }
                out
            }
            Formula::Literal(v) => *v,
            _ => f64::NAN,
        }
    }
}
impl<'a> From<&'a str> for Formula {
    fn from(input: &str) -> Self {
        if input.starts_with("=SUM(") && input.ends_with(')') {
            let range = &input[5..(input.len() - 1)];
            if let Some((l, r)) = range.split_once(':') {
                if let (Ok(l), Ok(r)) = (l.parse(), r.parse()) {
                    return Formula::Sum(l..=r);
                }
            }
        } else if let Ok(v) = input.parse::<f64>() {
            return Formula::Literal(v);
        }
        Formula::Unknown
    }
}

mod style {
    use async_ui_web::css;

    css!(
        "
.wrapper {
	overflow: scroll;
	max-height: 40em;
}
.table {
	table-layout: fixed;
	border-collapse: collapse;
}
.table tr, .table td {
	padding: 0;
}
.cell {
	position: relative;
}
.cell > span {
	position: absolute;
	top: 0;
	bottom: 0;
	margin: auto;
	right: 2px;
	pointer-events: none;
}
.cell:focus-within > span {
	visibility: hidden;
}
.cell > input {
	border: 1px solid grey;
}
		"
    );
}
