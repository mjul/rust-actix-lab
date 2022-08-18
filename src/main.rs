use actix::prelude::*;
use rust_actix_labx::{
    ObserveResponseTimeRequest, ResponseTime, ResponseTimeP95Request, WindowedPercentileService,
};
use time::{Date, Month, PrimitiveDateTime};

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

    // start new actor
    let addr = WindowedPercentileService::new(month_of_july).start();

    let rt_minutes_at = |rt: time::Duration, day, hh, mm| -> ResponseTime {
        let t = PrimitiveDateTime::new(
            Date::from_calendar_date(2022, Month::July, day).unwrap(),
            time::Time::from_hms(hh, mm, 0).unwrap(),
        );
        ResponseTime::new(rt, t)
    };

    for i in 0..100 {
        let req = ObserveResponseTimeRequest::new(rt_minutes_at(
            time::Duration::seconds(180 + i),
            1,
            12,
            (i % 60) as u8,
        ));
        let _ = addr.send(req).await;
    }

    let resp = addr.send(ResponseTimeP95Request::new()).await;
    let p95 = resp.unwrap();

    // handle() returns tokio handle
    println!("RESULT: {:?}", p95.value);

    // stop system and exit
    System::current().stop();
}
