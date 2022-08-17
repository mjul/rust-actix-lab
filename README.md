# Actors with Rust Actix
Experiments with actors using the Rust Actix library.

This library appears to be the most used actor framework for Rust.

# Scenario: Helpdesk Response Time Monitoring

We are collecting streams of observations over time, response time for helpdesk requests.
We want to compute percentiles over different time windows.
When the percentiles breach certain levels, we want to send a message.

