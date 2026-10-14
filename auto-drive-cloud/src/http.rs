use tokio::sync::broadcast::{Receiver, Sender};

use crate::{auto_drive_edge_service::v1::watch_the_beginner_car_events_response::Event, http::watch_logs::watch_logs_path};

pub mod watch_logs;

pub async fn http_server(receiver: Sender<Event>) {
    let server = tokio::spawn(async move {
        let watch_logs = watch_logs_path(receiver);
        let address = ([127, 0, 0, 1], 3030);

        println!("serving http to {address:?}");

        warp::serve(watch_logs).run(address).await;
    });

    // auto_drive_edge_server.run().await;

    // let counter = meter
    //     .u64_counter("battery.connections")
    //     .build();

    // let attrs: [KeyValue; 1] = [KeyValue::new("build", "build_a")];

    // counter.add(1, &[]);

    // 4. Create your tracer and start recording spans
    // let tracer = global::tracer("rust-grpc-service");

    // tracer.in_span("main-operation", |cx| {
    //     let span = cx.span();
    //     span.add_event("Doing some heavy work...", vec![]);
    // });

    server.await.unwrap();
}