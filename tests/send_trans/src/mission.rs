use config::{AppConfig, Node};
use execute::Mission;
use hyper::{self, Client};
use report::SimpleReport;
use std::thread;
use std::time::{Duration, Instant};
use transaction::make_tx_msg;

#[derive(Debug)]
pub struct MissionData {
    pub protocol: String,
    pub amount: usize,
    pub interval: usize,
    pub category: String,
}

impl MissionData {
    fn from(c: &AppConfig) -> MissionData {
        MissionData {
            protocol: c.protocol.clone(),
            amount: c.amount,
            interval: c.interval,
            category: c.category.clone(),
        }
    }
}

fn doing(node: &Node, data: &MissionData) -> SimpleReport {
    let url = format!("{}://{}:{}", data.protocol, node.host, node.port);
    let amount = data.amount;
    let interval = data.interval;
    let mut count = 0;
    let mut report = SimpleReport::new();
    let wait_millis = Duration::from_millis(data.interval as u64);
    let client = Client::new();
    loop {
        let now = Instant::now();
        if amount != 0 && count == amount {
            break; // TODO Catch Ctrl+C when no break
        }
        count += 1;
        let msg = make_tx_msg(&data.category, &node.key);
        report.add(now.elapsed(), post(&client, &url, &msg));
        if interval != 0 {
            thread::sleep(wait_millis);
        }
    }
    report
}

pub fn get_mission(config: &AppConfig) -> Mission<MissionData> {
    Mission {
        data: MissionData::from(config),
        doing: Box::new(doing),
    }
}

fn post(client: &Client, url: &str, msg: &str) -> (usize, usize, usize) {
    trace!("Send msg [{}] to [{}]", msg, url);
    client
        .post(url)
        .body(msg)
        .send()
        .and_then(|res| match res.status {
                      hyper::Ok => Ok((1, 0, 0)),
                      _ => Ok((0, 1, 0)),
                  })
        .unwrap_or((0, 0, 1))
}
