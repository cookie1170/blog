#[derive(Deserialize, PartialEq, Debug, Clone)]
#[serde(try_from = "DeserializedGraph")]
pub struct Graph {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: Option<f64>,
    pub max_y: Option<f64>,
    pub aspect: Option<f64>,
    pub expr: String,
    pub size: Size,
}

impl Graph {
    const NUM_SAMPLES: u32 = 256;
    const GRID_LINES_Y: usize = 10;
    const HEIGHT: u32 = 256;

    pub fn graph(&self) -> Result<String> {
        let mut buf = String::with_capacity(256);
        {
            let expr: meval::Expr = self.expr.parse()?;
            let expr = expr.bind("x").context("failed to evalute expression")?;
            let points = (0..=Self::NUM_SAMPLES)
                .map(|p| {
                    let t = p as f64 / Self::NUM_SAMPLES as f64;
                    let x = self.min_x + (self.max_x - self.min_x) * t;
                    let y = expr(x);
                    (x, y)
                })
                .collect::<Vec<_>>();

            let (min_y, max_y) = match points.iter().copied().minmax_by_key(|(_x, y)| *y) {
                MinMaxResult::NoElements | MinMaxResult::OneElement(..) => {
                    bail!("should have at least two points")
                }
                MinMaxResult::MinMax((_min_x, min_y), (_max_x, max_y)) => (min_y, max_y),
            };

            let (min_y, max_y) = (
                self.min_y.unwrap_or(f64::round(min_y * 10.0) / 10.0),
                self.max_y.unwrap_or(f64::round(max_y * 10.0) / 10.0),
            );
            let aspect = self
                .aspect
                .unwrap_or((self.max_x - self.min_x).abs() / (max_y - min_y).abs());
            // round it to 1 decimal place because the y may be a little off
            let width = (Self::HEIGHT as f64 * aspect) as u32;
            // take into account the rounding that occured
            let area = SVGBackend::with_string(&mut buf, (width, Self::HEIGHT)).into_drawing_area();

            let mut chart = ChartBuilder::on(&area)
                .build_cartesian_2d(self.min_x..self.max_x, min_y..max_y)
                .context("failed to build chart")?;

            chart
                .configure_mesh()
                .light_line_style(RGBAColor(0, 0, 0, 0.0))
                .bold_line_style(RGBAColor(128, 128, 128, 1.0))
                .x_labels((Self::GRID_LINES_Y as f64 * aspect) as usize)
                .y_labels(Self::GRID_LINES_Y)
                .draw()
                .context("failed to draw mesh to chart")?;

            let mut accum = Vec::with_capacity(Self::NUM_SAMPLES as usize + 1);

            let mut points = points.into_iter().peekable();

            // crimes to handle the graph going out-of-bounds
            while let Some((x, y)) = points.next() {
                if min_y <= y && y <= max_y {
                    accum.push((x, y));
                } else {
                    chart
                        .draw_series(LineSeries::new(accum.iter().copied(), WHITE))
                        .context("failed to draw chart")?;
                    accum.clear();

                    while let Some(&(_x, y)) = points.peek()
                        && !(min_y <= y && y <= max_y)
                    {
                        points.next();
                    }
                }
            }

            chart
                .draw_series(LineSeries::new(accum.iter().copied(), WHITE))
                .context("failed to draw chart")?;

            area.present()?;
        }

        Ok(buf)
    }
}

#[derive(Deserialize)]
struct DeserializedGraph {
    preset: Option<String>,
    x: Option<String>,
    y: Option<String>,
    aspect: Option<f64>,
    expr: String,
    #[serde(default)]
    size: Size,
}

impl TryFrom<DeserializedGraph> for Graph {
    type Error = anyhow::Error;

    fn try_from(value: DeserializedGraph) -> Result<Self> {
        let ((min_x, max_x), (min_y, max_y)) = {
            match value.preset.as_deref() {
                Some("unit") => ((0.0, 1.0), (None, None)),
                Some(other) => bail!("unknown preset '{other}'"),
                None => {
                    let x = value.x.context("x required with no preset")?;
                    let x = parse_range(&x).context("invalid x")?;
                    let mut y = (None, None);
                    if let Some(y_range) = value.y {
                        let (min_y, max_y) = parse_range(&y_range).context("invalid y")?;
                        y = (Some(min_y), Some(max_y));
                    }
                    (x, y)
                }
            }
        };
        let aspect = value.aspect;
        let expr = value.expr;
        let size = value.size;

        Ok(Self {
            min_x,
            max_x,
            min_y,
            max_y,
            aspect,
            expr,
            size,
        })
    }
}

fn parse_range(str: &str) -> Result<(f64, f64)> {
    let range = if let Ok(x) = str.parse::<f64>() {
        (-x, x)
    } else {
        let (min, max) = str.split_once("..").context("expected number or range")?;
        (
            min.parse().context("invalid number")?,
            max.parse().context("invalid number")?,
        )
    };
    Ok(range)
}

use crate::Size;
use anyhow::{Context, Result, bail};
use itertools::{Itertools as _, MinMaxResult};
use plotters::{
    backend::SVGBackend,
    chart::ChartBuilder,
    drawing::IntoDrawingArea,
    series::LineSeries,
    style::{RGBAColor, full_palette::WHITE},
};
use serde::Deserialize;
