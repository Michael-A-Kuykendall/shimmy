// WebSocket console endpoint integration test
// Tests that ws://localhost:11435/ws/console accepts connections and streams tokens

#[cfg(feature = "console")]
#[tokio::test]
async fn test_console_websocket_endpoint() {
    use futures_util::{SinkExt, StreamExt};
    use std::time::Duration;
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    // Connect to shimmy's WebSocket console endpoint
    let ws_url = "ws://localhost:11435/ws/console";

    println!("Attempting to connect to {}", ws_url);

    let connect_result = tokio::time::timeout(Duration::from_secs(5), connect_async(ws_url)).await;

    match connect_result {
        Ok(Ok((ws_stream, _))) => {
            println!("✅ WebSocket connected to /ws/console");

            let (mut write, mut read) = ws_stream.split();

            // Send a test message
            let test_prompt = "Say 'test' and nothing else";
            write
                .send(Message::Text(test_prompt.to_string()))
                .await
                .unwrap();
            println!("📤 Sent test prompt: {}", test_prompt);

            // Receive streaming tokens
            let mut received_tokens = Vec::new();
            let mut done = false;

            while !done {
                if let Ok(Some(msg)) =
                    tokio::time::timeout(Duration::from_secs(30), read.next()).await
                {
                    match msg {
                        Ok(Message::Text(text)) => {
                            if text == r#"{"done":true}"# {
                                println!("✅ Received done signal");
                                done = true;
                            } else {
                                received_tokens.push(text.clone());
                                print!("{}", text);
                            }
                        }
                        Ok(Message::Close(_)) => {
                            println!("\n✅ WebSocket closed cleanly");
                            break;
                        }
                        Err(e) => {
                            eprintln!("\n❌ WebSocket error: {}", e);
                            break;
                        }
                        _ => {}
                    }
                } else {
                    println!("\n⚠️  Timeout waiting for tokens");
                    break;
                }
            }

            println!("\n📊 Received {} tokens", received_tokens.len());
            assert!(
                !received_tokens.is_empty(),
                "Should receive at least one token"
            );

            println!("✅ WebSocket endpoint fully functional - streaming tokens work!");
        }
        Ok(Err(e)) => {
            println!("❌ Failed to connect to WebSocket: {}", e);
            println!("⚠️  Make sure shimmy server is running with --features console");
            println!("   Command: cargo run --bin shimmy --features \"llama,console\" -- serve --bind 127.0.0.1:11435");
            panic!(
                "WebSocket connection failed - server not running or console feature not enabled"
            );
        }
        Err(_) => {
            println!("❌ Connection attempt timed out");
            panic!("WebSocket connection timeout - server not responding");
        }
    }
}
