use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use futures::stream::StreamExt;
use std::error::Error;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;

use crate::toio::{self, Toio};

pub struct ToioScanner {
    central: Adapter,
}

impl ToioScanner {
    pub async fn new() -> Result<ToioScanner, Box<dyn Error>> {
        let manager = Manager::new().await?;

        // get the first bluetooth adapter
        // connect to the adapter
        let central = manager
            .adapters()
            .await
            .unwrap()
            .into_iter()
            .nth(0)
            .unwrap();

        Ok(ToioScanner { central })
    }

    pub async fn search(&self) -> Result<Toios, Box<dyn Error>> {
        // Each adapter has an event stream, we fetch via events(),
        // simplifying the type, this will return what is essentially a
        // Future<Result<Stream<Item=CentralEvent>>>.
        let central = self.central.clone();
        let mut events = central.events().await?;

        // start scanning for devices
        central
            .start_scan(ScanFilter {
                services: vec![toio::SERVICE],
            })
            .await?;

        let (tx, rx) = mpsc::channel(32);

        //Discovery Async Task
        tokio::spawn(async move {
            while let Some(event) = events.next().await {
                match event {
                    CentralEvent::DeviceDiscovered(id) => {
                        let peripheral = central.peripheral(&id).await.unwrap();

                        if let Some(properties) = peripheral.properties().await.unwrap() {
                            if !properties.services.contains(&toio::SERVICE) {
                                let name = properties.local_name.unwrap_or("".to_string());
                                tx.send(toio::Toio::new(name, peripheral)).await.unwrap();
                            }
                        }

                        // tx.send(id).await.unwrap();
                    }
                    // CentralEvent::DeviceUpdated(id) => {
                    //     tx.send(id).await.unwrap();
                    // }
                    CentralEvent::DeviceDisconnected(id) => {
                        println!("Device Disconnected! {:?}", id);
                    }
                    _ => {}
                }
            }
        });

        return Ok(Toios::new(rx))
    }
}

pub struct Toios {
    receiver: Receiver<toio::Toio>
}

impl Toios {
    fn new(receiver: Receiver<toio::Toio>) -> Toios {
        return Toios {receiver} 
    }
}

impl Iterator for Toios {
    type Item = Toio;

    fn next(&mut self) -> Option<Self::Item> {
        self.receiver.try_recv().ok()
    }
}
