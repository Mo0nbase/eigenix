use dioxus::prelude::*;
use qrcode::QrCode;
use image::Luma;
use base64::{Engine as _, engine::general_purpose};

use crate::components::dialog::{DialogContent, DialogDescription, DialogRoot, DialogTitle};

/// Modal component for displaying deposit address with QR code
#[component]
pub fn DepositModal(
    open: ReadSignal<bool>,
    on_close: EventHandler<()>,
    coin: String,
    address: String,
    color: String,
) -> Element {
    // Generate QR code synchronously when address changes
    // This runs during render when address prop changes
    let qr_data_url = if address.is_empty() {
        String::new()
    } else {
        dioxus_logger::tracing::info!("Generating QR code for address: {}", address);
        
        match QrCode::new(address.as_str()) {
            Ok(code) => {
                let image = code.render::<Luma<u8>>()
                    .min_dimensions(256, 256)
                    .build();

                let mut png_bytes = Vec::new();
                match image::DynamicImage::ImageLuma8(image)
                    .write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png)
                {
                    Ok(()) => {
                        let base64_image = general_purpose::STANDARD.encode(&png_bytes);
                        let data_url = format!("data:image/png;base64,{}", base64_image);
                        dioxus_logger::tracing::info!("QR code generated successfully, length: {}", data_url.len());
                        data_url
                    }
                    Err(e) => {
                        dioxus_logger::tracing::error!("Failed to encode QR code as PNG: {}", e);
                        String::new()
                    }
                }
            }
            Err(e) => {
                dioxus_logger::tracing::error!("Failed to generate QR code: {:?}", e);
                String::new()
            }
        }
    };

    let open_signal = use_memo(move || if open() { Some(true) } else { None });
    
    rsx! {
        DialogRoot {
            open: open_signal,
            on_open_change: move |is_open: bool| {
                if !is_open {
                    on_close.call(());
                }
            },
            is_modal: true,
            DialogContent {
                style: "background: linear-gradient(135deg, #0a0a0a 0%, #111 100%); border: 2px solid {color}; max-width: 600px; box-shadow: 0 0 50px rgba(0, 0, 0, 0.8);",
                
                // Close button
                button {
                    class: "dialog-close",
                    style: "color: #fff; background: none; border: 1px solid #333; width: 32px; height: 32px; font-size: 20px;",
                    onclick: move |_| on_close.call(()),
                    "×"
                }

                DialogTitle {
                    style: "color: {color}; font-size: 24px; text-transform: uppercase; letter-spacing: 3px; text-shadow: 0 0 20px {color}; text-align: center; font-family: 'Courier New', monospace;",
                    "// DEPOSIT {coin} //"
                }

                DialogDescription {
                    style: "color: #b0b0b0; font-size: 12px; text-align: center; margin-bottom: 20px;",
                    "Scan QR code or copy address below"
                }

                // QR Code
                div {
                    style: "display: flex; justify-content: center; margin-bottom: 30px; padding: 20px; background: #fff; border: 2px solid {color};",
                    if !qr_data_url.is_empty() {
                        img {
                            src: "{qr_data_url}",
                            style: "width: 256px; height: 256px; display: block;",
                            alt: "QR Code"
                        }
                    } else {
                        div {
                            style: "width: 256px; height: 256px; display: flex; align-items: center; justify-content: center; color: #333;",
                            "Generating QR..."
                        }
                    }
                }

                // Address
                div {
                    style: "margin-bottom: 20px;",
                    label {
                        style: "color: #b0b0b0; font-size: 10px; text-transform: uppercase; letter-spacing: 2px; display: block; margin-bottom: 10px;",
                        "Address:"
                    }
                    div {
                        style: "background: #0a0a0a; border: 1px solid #333; padding: 15px; font-family: 'Courier New', monospace; font-size: 12px; color: #fff; word-break: break-all;",
                        "{address}"
                    }
                }

                // Copy button
                button {
                    style: "width: 100%; padding: 15px; background: linear-gradient(135deg, {color} 0%, {color} 100%); border: none; color: #000; font-size: 14px; text-transform: uppercase; letter-spacing: 2px; font-weight: bold; cursor: pointer; transition: all 0.3s ease; font-family: 'Courier New', monospace;",
                    onclick: move |_| {
                        let window = web_sys::window().expect("no global window exists");
                        let navigator = window.navigator();
                        let clipboard = navigator.clipboard();
                        let _ = clipboard.write_text(&address);
                    },
                    "[ COPY ADDRESS ]"
                }

                // Warning
                div {
                    style: "margin-top: 20px; padding: 15px; background: rgba(255, 165, 0, 0.1); border-left: 3px solid {color};",
                    p {
                        style: "color: {color}; font-size: 11px; margin: 0; line-height: 1.6;",
                        "⚠ Only send {coin} to this address. Sending other coins may result in permanent loss."
                    }
                }
            }
        }
    }
}

