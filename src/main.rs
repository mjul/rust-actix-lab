use actix::prelude::*;
use rust_actix_labx::{
    CalculateResponseTimeStatistics, ObserveResponseTime, ResponseTimeMonitoringPipeline,
    ResponseTimeObservation, WindowedPercentileService,
};
use time::{Date, Duration, Month, PrimitiveDateTime};

// This macro sets up the Actix system.
// Otherwise you would have to call:
//
//  let system = actix::System::new();
//  let _ = system.run();
//
// See https://github.com/actix/actix
#[actix_rt::main]
async fn main() {
    println!("Starting...");

    let jul1 = Date::from_calendar_date(2022, Month::July, 1).unwrap();
    let aug1 = Date::from_calendar_date(2022, Month::August, 1).unwrap();
    let midnight = time::Time::MIDNIGHT;

    let month_of_july = rust_actix_labx::Interval::new(
        PrimitiveDateTime::new(jul1, midnight),
        PrimitiveDateTime::new(aug1, midnight),
    );

    //
    // Warm-up exercise:
    // Try calling the individual actors
    //
    let addr = WindowedPercentileService::new(month_of_july).start();

    let rt_minutes_at = |rt: Duration, day, hh, mm| -> ResponseTimeObservation {
        let t = PrimitiveDateTime::new(
            Date::from_calendar_date(2022, Month::July, day).unwrap(),
            time::Time::from_hms(hh, mm, 0).unwrap(),
        );
        ResponseTimeObservation::new(rt, t)
    };

    for i in 0..100 {
        let req = ObserveResponseTime::new(rt_minutes_at(
            Duration::seconds(180 + i),
            1,
            12,
            (i % 60) as u8,
        ));
        let _ = addr.send(req).await;
    }

    let resp = addr.send(CalculateResponseTimeStatistics::new()).await;
    let stats = resp.unwrap().value;

    // handle() returns tokio handle
    println!(
        "RESULT: P95 = {}s  (n={})",
        stats.p95.unwrap().whole_seconds(),
        stats.n
    );

    //
    // Now, try calling the pipeline actor that orchestrates everything
    //

    let month_of_july = rust_actix_labx::Interval::new(
        PrimitiveDateTime::new(jul1, midnight),
        PrimitiveDateTime::new(aug1, midnight),
    );

    let pipeline_addr =
        ResponseTimeMonitoringPipeline::new(month_of_july, Duration::minutes(3)).start();

    for i in 0..10 {
        let req = ObserveResponseTime::new(rt_minutes_at(
            Duration::seconds(180 + i),
            1,
            12,
            (i % 60) as u8,
        ));
        let _ = pipeline_addr.send(req).await;
    }

    let resp = pipeline_addr
        .send(CalculateResponseTimeStatistics::new())
        .await;
    let stats = resp.unwrap().value;

    match stats.p95 {
        Some(p95) => {
            println!("RESULT: P95 = {}s  (n={})", p95.whole_seconds(), stats.n);
        }
        None => {
            println!("RESULT: P95 not defined.");
        }
    }

    // stop system and exit
    System::current().stop();
}
