use actix::prelude::*;
use time::{Duration, PrimitiveDateTime};

/// A half-open time interval. From is included, to is not.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Interval {
    from: PrimitiveDateTime,
    to: PrimitiveDateTime,
}
impl Interval {
    pub fn new(from: PrimitiveDateTime, to: PrimitiveDateTime) -> Self {
        Self { from, to }
    }
    fn includes(&self, t: &PrimitiveDateTime) -> bool {
        (&self.from <= t) && (t < &self.to)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseTime {
    value: Duration,
    time: PrimitiveDateTime,
}

impl ResponseTime {
    /// Construct a response time measurement with the given response time value observed at the given time.
    pub fn new(value: Duration, time: PrimitiveDateTime) -> ResponseTime {
        ResponseTime { value, time }
    }
}

/// A windowed response time percentile calculator
pub struct WindowedPercentileService {
    window: Interval,
    qualified_observations: Vec<Duration>,
}

/// Get the value of the 95th percentile
fn p95<T: Ord + Clone>(data: &Vec<T>) -> Option<T> {
    let n = &data.len();
    match n {
        0 => None,
        _ => {
            let n5 = (5 * n) / 100;
            let n95 = (n - n5 - 1).max(0);
            let mut sorted: Vec<T> = data.clone();
            sorted.sort();
            Some(sorted[n95].clone())
        }
    }
}

impl WindowedPercentileService {
    // Create a new Windowed Percentile Service for a given time-window.
    pub fn new(window: Interval) -> Self {
        WindowedPercentileService {
            window,
            qualified_observations: vec![],
        }
    }
    fn add_observation(&mut self, datum: &ResponseTime) {
        if self.window.includes(&datum.time) {
            self.qualified_observations.push(datum.value);
        }
    }
    fn get_p95(&self) -> Option<Duration> {
        p95(&self.qualified_observations)
    }
}

// This makes it an Actor
impl Actor for WindowedPercentileService {
    type Context = Context<Self>;
}

// Messages can be declared with macros or by implementing the right traits

/// Observed 95th percentile for response time
#[derive(Debug, MessageResponse)]
pub struct ResponseTimeP95Response {
    pub value: Option<Duration>,
    pub interval: Interval,
}

#[derive(Message)]
#[rtype(result = "ResponseTimeP95Response")]
pub struct ResponseTimeP95Request {}

impl ResponseTimeP95Request {
    /// Create a new message to request the response time P95 level.
    pub fn new() -> ResponseTimeP95Request {
        ResponseTimeP95Request {}
    }
}

impl Default for ResponseTimeP95Request {
   fn default() -> Self {
        Self::new()
    }
}


// by implementing handler, we declare that the service accepts this message

impl Handler<ResponseTimeP95Request> for WindowedPercentileService {
    type Result = ResponseTimeP95Response;
    fn handle(&mut self, _msg: ResponseTimeP95Request, _ctx: &mut Context<Self>) -> Self::Result {
        ResponseTimeP95Response {
            value: self.get_p95(),
            interval: self.window.clone(),
        }
    }
}

// Implement a message with traits
pub struct ObserveResponseTimeRequest {
    datum: ResponseTime,
}
impl ObserveResponseTimeRequest {
    pub fn new(rt: ResponseTime) -> ObserveResponseTimeRequest {
        ObserveResponseTimeRequest { datum: rt }
    }
}

impl Message for ObserveResponseTimeRequest {
    // unit () result, no return message
    type Result = ();
}

/// Collect an observation
impl Handler<ObserveResponseTimeRequest> for WindowedPercentileService {
    type Result = ();
    fn handle(
        &mut self,
        msg: ObserveResponseTimeRequest,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        self.add_observation(&msg.datum);
    }
}

#[cfg(test)]
mod tests {
    use time::{PrimitiveDateTime, Date, Month, Time};

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    fn today_at(hour: u8) -> PrimitiveDateTime {
        let today: Date =
            Date::from_calendar_date(2022, Month::August, 17).unwrap();
        PrimitiveDateTime::new(today, Time::from_hms(hour, 0, 0).unwrap())
    }

    #[test]
    fn interval_includes() {
        let i = Interval::new(today_at(9), today_at(17));

        assert_eq!(false, i.includes(&today_at(8)));
        assert_eq!(true, i.includes(&today_at(9)));
        assert_eq!(true, i.includes(&today_at(16)));
        assert_eq!(false, i.includes(&today_at(17)));
    }

    #[test]
    fn p95_empty_int() {
        let data: Vec<i32> = vec![];
        let actual = p95(&data);
        assert_eq!(None, actual);
    }

    #[test]
    fn p95_single() {
        let data: Vec<i32> = vec![1];
        let actual = p95(&data);
        assert_eq!(Some(1), actual);
    }

    #[test]
    fn p95_constant_values() {
        let data: Vec<i32> = (0..100).map(|_| 42).collect();
        let actual = p95(&data);
        assert_eq!(Some(42), actual);
    }

    #[test]
    fn p95_range100() {
        let data: Vec<i32> = (1..101).collect();
        let actual = p95(&data);
        assert_eq!(Some(95), actual);
    }

    #[test]
    fn p95_range1000() {
        let data: Vec<i32> = (1..1001).collect();
        let actual = p95(&data);
        assert_eq!(Some(950), actual);
    }
}
