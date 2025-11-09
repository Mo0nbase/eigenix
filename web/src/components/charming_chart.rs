use dioxus::prelude::*;

use charming::{
    component::{Axis, Grid},
    element::{AxisLabel, AxisLineStyle, AxisType, Color, LineStyle, SplitLine, Tooltip, Trigger},
    series::Line,
    Chart, WasmRenderer,
};

use crate::types::metrics::MetricValue;

/// Reusable chart component for displaying time-series metric data using Charming (ECharts)
///
/// This component renders an interactive line chart that automatically updates when data changes.
#[component]
pub fn CharmingChart(
    /// Unique HTML id for the chart container
    id: String,
    /// Chart title to display above the chart
    title: String,
    /// Time-series data points to plot
    data: Vec<MetricValue>,
    /// Border color for the line (CSS color string)
    color: String,
    /// Whether the Y-axis should begin at zero
    y_begin_at_zero: bool,
) -> Element {
    let chart_id = id.clone();
    let chart_data = data.clone();
    let chart_color = color.clone();

    // Create renderer with fixed dimensions
    let renderer = use_signal(|| WasmRenderer::new(600, 300));

    // Render chart whenever data changes
    use_effect(move || {
        if chart_data.is_empty() {
            return;
        }

        // Extract values and format timestamps
        let labels: Vec<String> = chart_data
            .iter()
            .map(|d| {
                // Parse ISO timestamp and format as HH:MM or MM-DD HH:MM
                if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(&d.timestamp) {
                    // If all data is from today, show just time; otherwise show date + time
                    if chart_data.len() > 1 {
                        let first = chrono::DateTime::parse_from_rfc3339(&chart_data[0].timestamp).ok();
                        let last = chrono::DateTime::parse_from_rfc3339(&chart_data[chart_data.len()-1].timestamp).ok();
                        if let (Some(f), Some(l)) = (first, last) {
                            let duration = l.signed_duration_since(f);
                            if duration.num_hours() > 24 {
                                // Show date and time for longer ranges
                                return parsed.format("%m-%d %H:%M").to_string();
                            }
                        }
                    }
                    // Show just time for short ranges
                    parsed.format("%H:%M").to_string()
                } else {
                    d.timestamp.clone()
                }
            })
            .collect();
        
        let values: Vec<f64> = chart_data.iter().map(|d| d.value).collect();

        // Parse color string to Color type
        let color: Color = chart_color.as_str().into();

        // Build the chart with improved styling
        let mut chart = Chart::new()
            .grid(
                Grid::new()
                    .left("10%")
                    .right("5%")
                    .top("10%")
                    .bottom("20%")
                    .contain_label(true)
            )
            .tooltip(
                Tooltip::new()
                    .trigger(Trigger::Axis)
            )
            .x_axis(
                Axis::new()
                    .type_(AxisType::Category)
                    .data(labels)
                    .axis_label(
                        AxisLabel::new()
                            .rotate(45)
                            .color("#b0b0b0")
                            .font_size(10)
                    )
                    .axis_line(
                        charming::element::AxisLine::new()
                            .line_style(AxisLineStyle::from((1.0, "#333")))
                    )
                    .split_line(
                        SplitLine::new()
                            .show(false)
                    )
            )
            .y_axis(
                Axis::new()
                    .type_(AxisType::Value)
                    .axis_label(
                        AxisLabel::new()
                            .color("#b0b0b0")
                            .font_size(10)
                    )
                    .axis_line(
                        charming::element::AxisLine::new()
                            .line_style(AxisLineStyle::from((1.0, "#333")))
                    )
                    .split_line(
                        SplitLine::new()
                            .show(true)
                            .line_style(LineStyle::new().color("#222").opacity(0.3))
                    )
            )
            .series(
                Line::new()
                    .data(values)
                    .line_style(LineStyle::new().color(color.clone()).width(2))
                    .item_style(charming::element::ItemStyle::new().color(color))
                    .smooth(true)
                    .symbol_size(4)
            );

        // Configure Y-axis to start at zero if requested
        if y_begin_at_zero {
            chart = chart.y_axis(
                Axis::new()
                    .type_(AxisType::Value)
                    .min(0.0)
                    .axis_label(
                        AxisLabel::new()
                            .color("#b0b0b0")
                            .font_size(10)
                    )
                    .axis_line(
                        charming::element::AxisLine::new()
                            .line_style(AxisLineStyle::from((1.0, "#333")))
                    )
                    .split_line(
                        SplitLine::new()
                            .show(true)
                            .line_style(LineStyle::new().color("#222").opacity(0.3))
                    )
            );
        }

        // Render the chart
        if let Err(e) = renderer.read_unchecked().render(&chart_id, &chart) {
            dioxus_logger::tracing::error!("Failed to render chart {}: {:?}", chart_id, e);
        }
    });

    rsx! {
        div {
            style: "width: 100%; background: linear-gradient(135deg, #111 0%, #0a0a0a 100%); border: 1px solid #333; padding: 20px; position: relative; transition: all 0.3s ease; margin-bottom: 20px;",
            
            div {
                style: "position: absolute; top: 0; left: 0; right: 0; height: 2px; background: linear-gradient(90deg, transparent, #fff, transparent); opacity: 0.3;"
            }
            
            h3 {
                style: "color: #fff; margin: 0 0 15px 0; font-size: 12px; text-align: left; text-transform: uppercase; letter-spacing: 2px; font-family: 'Courier New', monospace;",
                "[ {title} ]"
            }
            div { 
                id: "{id}",
                style: "width: 100%; display: flex; justify-content: center; filter: contrast(1.1);"
            }
        }
    }
}

