use dioxus::prelude::*;

use charming::{
    component::Axis,
    element::{AxisType, Color, LineStyle},
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
    let chart_title = title.clone();

    // Create renderer with fixed dimensions
    let renderer = use_signal(|| WasmRenderer::new(600, 300));

    // Render chart whenever data changes
    use_effect(move || {
        if chart_data.is_empty() {
            return;
        }

        // Extract labels (timestamps) and values
        let labels: Vec<String> = chart_data.iter().map(|d| d.timestamp.clone()).collect();
        let values: Vec<f64> = chart_data.iter().map(|d| d.value).collect();

        // Parse color string to Color type
        let color: Color = chart_color.as_str().into();

        // Build the chart
        let mut chart = Chart::new()
            .x_axis(
                Axis::new()
                    .type_(AxisType::Category)
                    .data(labels)
                    .axis_label(charming::element::AxisLabel::new().rotate(45))
            )
            .y_axis(
                Axis::new()
                    .type_(AxisType::Value)
            )
            .series(
                Line::new()
                    .data(values)
                    .line_style(LineStyle::new().color(color))
                    .smooth(true)
            );

        // Add title if provided
        if !chart_title.is_empty() {
            chart = chart.title(charming::component::Title::new().text(&chart_title));
        }

        // Configure Y-axis to start at zero if requested
        if y_begin_at_zero {
            chart = chart.y_axis(
                Axis::new()
                    .type_(AxisType::Value)
                    .min(0.0)
            );
        }

        // Render the chart
        if let Err(e) = renderer.read_unchecked().render(&chart_id, &chart) {
            dioxus_logger::tracing::error!("Failed to render chart {}: {:?}", chart_id, e);
        }
    });

    rsx! {
        div {
            style: "width: 100%; background: #222; border-radius: 8px; padding: 15px; box-shadow: 0 2px 4px rgba(0,0,0,0.2);",
            h3 {
                style: "color: #ccc; margin: 0 0 10px 0; font-size: 1.1rem; text-align: center;",
                "{title}"
            }
            div { 
                id: "{id}",
                style: "width: 100%; display: flex; justify-content: center;"
            }
        }
    }
}

