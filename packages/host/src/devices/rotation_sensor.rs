use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::{sync::Mutex, task::AbortHandle, time::sleep};
use vex_v5_qemu_protocol::{
    rotation_sensor::{RotationSensorData, RotationObject},
    SmartPortData,
};

use crate::peripherals::smartport::SmartPort;

#[derive(Debug)]
pub struct RotationSensor {
    task: AbortHandle,
    data: Arc<Mutex<RotationSensorData>>,
}

impl RotationSensor {
    // TODO: determine this
    pub const UPDATE_INTERVAL: Duration = Duration::from_millis(10);

    pub fn new(mut port: SmartPort) -> Self {
        let start = Instant::now();
        let data = Arc::new(Mutex::new(RotationSensorData {
            object: None,
            status: 0,
        }));

        Self {
            data: data.clone(),
            task: tokio::task::spawn(async move {
                loop {
                    port.send(
                        SmartPortData::RotationSensor(data.lock().await.clone()),
                        start.elapsed().as_millis() as u32,
                    )
                    .await;

                    sleep(Self::UPDATE_INTERVAL).await;
                }
            })
            .abort_handle(),
        }
    }

    pub async fn set_object(&mut self, object: Option<RotationObject>) {
        self.data.lock().await.object = object;
    }

    pub async fn set_status(&mut self, status: u32) {
        self.data.lock().await.status = status;
    }
}

impl Drop for RotationSensor {
    fn drop(&mut self) {
        self.task.abort();
    }
}
