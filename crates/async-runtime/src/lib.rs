use tokio::select;
use tokio::sync::mpsc::Receiver;

#[allow(unused)]
async fn consumer(mut rx_one: Receiver<u8>, mut rx_two: Receiver<u8>) -> Vec<u8> {
    let mut results = Vec::new();

    loop {
        select! {
            Some(value) = rx_one.recv() => {
                results.push(value);
            }
            Some(value) = rx_two.recv() => {
                results.push(value);
            }
            // Both channels are closed.
            else => {
                break;
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use tokio::spawn;
    use tokio::sync::mpsc::channel;

    use super::*;

    #[tokio::test]
    async fn ar_select_values() {
        let (tx_one, rx_one) = channel(10);
        let (tx_two, rx_two) = channel(10);

        let handle_one = spawn(async move {
            tx_one.send(1).await.expect("[TX_ONE] channel closed");
        });

        let handle_two = spawn(async move {
            tx_two.send(2).await.expect("[TX_TWO] channel closed");
        });

        let mut received = consumer(rx_one, rx_two).await;

        handle_one
            .await
            .expect("[HANDLE_ONE] panic in green thread");
        handle_two
            .await
            .expect("[HANDLE_TWO] panic in green thread");

        received.sort();
        assert_eq!(received, vec![1, 2]);
    }

    #[tokio::test]
    async fn ar_multiple_closed_channels() {
        let (tx_one, rx_one) = channel(10);
        let (tx_two, rx_two) = channel(10);

        // Close channels so consumer can break on empty.
        drop(tx_one);
        drop(tx_two);

        let received = consumer(rx_one, rx_two).await;

        assert!(received.is_empty());
    }
}
