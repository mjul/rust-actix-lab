use actix::prelude::*;
use rust_actix_labx;

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

    let jul1 = time::Date::from_calendar_date(2022, time::Month::July, 1).unwrap();
    let aug1 = time::Date::from_calendar_date(2022, time::Month::August, 1).unwrap();
    let midnight = time::Time::MIDNIGHT;

    let month_of_july = rust_actix_labx::Interval::new(
        time::PrimitiveDateTime::new(jul1, midnight),
        time::PrimitiveDateTime::new(aug1, midnight),
    );

    // start new actor
    let addr = rust_actix_labx::WindowedPercentileService::new(month_of_july).start();

    let rt_minutes_at = |rt: time::Duration, day, hh, mm| -> rust_actix_labx::ResponseTime {
        let t = time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2022, time::Month::July, day).unwrap(),
            time::Time::from_hms(hh, mm, 0).unwrap(),
        );
        rust_actix_labx::ResponseTime::new(rt, t)
    };

    for i in 0..100 {
        let req = rust_actix_labx::ObserveResponseTimeRequest::new(rt_minutes_at(
            time::Duration::seconds(180 + i),
            1,
            12,
            (i % 60) as u8,
        ));
        let _ = addr.send(req).await;
    }

    let resp = addr
        .send(rust_actix_labx::ResponseTimeP95Request::new())
        .await;
    let p95 = resp.unwrap();

    // handle() returns tokio handle
    println!("RESULT: {:?}", p95.value);

    // stop system and exit
    System::current().stop();
}
