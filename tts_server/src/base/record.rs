use prettytable::{format, Cell, Row, Table};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

// 结构体用于记录查询信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRecord {
    pub text: String,
    pub query_time: String,
    pub duration: Duration,
}

// 管理查询记录的结构体
#[derive(Default, Serialize, Deserialize)]
pub struct QueryTracker {
    pub total_cnts: usize,
    pub total_words: usize,
    pub first_launch_time: String,
    pub last_launch_time: String,
    pub records: VecDeque<QueryRecord>,
    pub cost_records: VecDeque<QueryRecord>,
}

pub const QUERY_FILE_NAME: &str = "./records/query.json";

impl QueryTracker {
    pub fn new(start_time: String) -> QueryTracker {
        let tracker = QueryTracker::load_from_file();
        if let Ok(mut t) = tracker {
            t.last_launch_time = start_time;
            t
        } else {
            Self {
                total_cnts: 0,
                total_words: 0,
                first_launch_time: start_time.clone(),
                last_launch_time: start_time.clone(),
                ..Default::default()
            }
        }
    }

    // 记录查询信息
    pub fn record_query(&mut self, text: String, query_time: String, duration: Duration) {
        let record = QueryRecord {
            text: text.clone(),
            query_time,
            duration,
        };

        // 保留最近10次查询记录
        if self.records.len() >= 10 {
            self.records.pop_back();
        }

        // 更新最耗时的查询记录
        if self.cost_records.len() >= 10
            && duration > self.cost_records[self.cost_records.len() - 1].duration
        {
            self.cost_records.pop_back();
        }

        if self.cost_records.len() < 10 {
            self.cost_records.push_back(record.clone());
            self.cost_records
                .make_contiguous()
                .sort_by(|a, b| b.duration.cmp(&a.duration));
        }

        self.records.push_front(record);
        self.total_cnts += 1;

        // 匹配中文字符的正则表达式
        let chinese_regex = Regex::new(r"[\u4e00-\u9fa5]").unwrap();
        let chinese_count = chinese_regex.find_iter(&text).count();

        // 匹配英文单词的正则表达式
        let english_regex = Regex::new(r"\b[a-zA-Z]+\b").unwrap();
        let english_count = english_regex.find_iter(&text).count();

        // 匹配数字的正则表达式
        let number_regex = Regex::new(r"\d+").unwrap();
        let number_count = number_regex.find_iter(&text).count();

        self.total_words += chinese_count + english_count + number_count;

        let _ = self.save_to_file();
    }

    // 获取查询记录
    pub fn get_records(&self) -> Vec<QueryRecord> {
        self.records.iter().cloned().collect()
    }

    pub fn save_to_file(&self) -> std::io::Result<()> {
        let serialized_data = serde_json::to_string(self)?;
        create_dir_all(Path::new(QUERY_FILE_NAME).parent().unwrap())?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(QUERY_FILE_NAME)?;
        file.write_all(serialized_data.as_bytes())?;
        Ok(())
    }

    pub fn load_from_file() -> std::io::Result<Self> {
        let mut file = File::open(QUERY_FILE_NAME)?;
        let mut serialized_data = String::new();
        file.read_to_string(&mut serialized_data)?;
        let tracker: QueryTracker = serde_json::from_str(&serialized_data)?;
        Ok(tracker)
    }

    pub fn to_table_string(&self) -> String {
        let mut table1 = Table::new();
        table1.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        table1.add_row(Row::new(vec![
            Cell::new("Index"),
            Cell::new("Time"),
            Cell::new("Duration (s)"),
            Cell::new("Text"),
        ]));

        for (index, record) in self.records.iter().enumerate() {
            table1.add_row(Row::new(vec![
                Cell::new(&index.to_string()),
                Cell::new(&record.query_time),
                Cell::new(&(record.duration.as_millis() as f64 / 1000.0).to_string()),
                Cell::new(&record.text),
            ]));
        }

        let mut table2 = Table::new();
        table2.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        table2.add_row(Row::new(vec![
            Cell::new("Index"),
            Cell::new("Time"),
            Cell::new("Duration (s)"),
            Cell::new("Text"),
        ]));

        for (index, record) in self.cost_records.iter().enumerate() {
            table2.add_row(Row::new(vec![
                Cell::new(&index.to_string()),
                Cell::new(&record.query_time),
                Cell::new(&(record.duration.as_millis() as f64 / 1000.0).to_string()),
                Cell::new(&record.text),
            ]));
        }

        format!("First launch at: {}\nLast launch at: {}\nTotal Query Times:{}\nTotal Query Words:{}\n\nLast 10 Queries:\n{}\n\nCost 10 Queries:\n{}",
            self.first_launch_time, self.last_launch_time, self.total_cnts, self.total_words, table1, table2)
    }
}
