use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::time::SystemTime;

fn read_entire_file(file_path: &str) -> Result<File, io::Error> {
    let file_res = match File::open(&file_path) {
        Ok(file) => Ok(file),
        Err(error) => {
            println!("ERROR: Unable to open file: {} : {:?}", file_path, error);
            Err(error)
        },
    };
    file_res
}

#[derive(Default, Clone)]
struct FreqKV {
    key: String,
    value: u32,
}

struct FreqKVs {
    freq_table: Vec<FreqKV>,
    count: usize,
}

impl FreqKVs {
   fn new() -> Self {
       Self {
           freq_table: Vec::new(),
           count: 0,
       }
   }

   fn find_key(&mut self, word: &str) -> Option<&mut FreqKV> { 
       let needle = word;
       for i in 0..self.count {
           let current_needle = &self.freq_table[i].key;
           if current_needle == needle {
               return Some(&mut self.freq_table[i]);
           }
       }
       None
   }

   fn append(&mut self, new_item: FreqKV) {
       self.freq_table.push(new_item);
   }

   fn sort_freq_table(&mut self) {
       self.freq_table.sort_by_key(|entry| std::cmp::Reverse(entry.value));
   }
}

fn naive_analysis(freq: &mut FreqKVs, words: &Vec<&str>) {
    let start_time = SystemTime::now();
    // Linear Search of forming the frequency table
    for token in words {
        if let Some(fkv) = freq.find_key(token) {
            fkv.value += 1;
        } else {
            let new_item = FreqKV { key: String::from(*token), value: 1 };
            freq.append(new_item);
            freq.count += 1;
        }
    }
    let end_time = SystemTime::now();
    let elapsed_time = end_time.duration_since(start_time).unwrap();

    println!("    Tokens: {}", freq.freq_table.len());
    freq.sort_freq_table();
    println!("    Top 10 tokens evaluated");
    for i in 0..10 {
        println!("      {}: ({}, {})", i, freq.freq_table[i].key, freq.freq_table[i].value);
    }
    println!("    Elapsed time: {:.3}s", elapsed_time.as_secs_f64());
}

pub trait Hashable {
    fn hash(&self) -> usize;
}

impl Hashable for String {
    fn hash(&self) -> usize {
        // djb2: http://www.cse.yorku.ca/~oz/hash.html
        let mut result: usize = 5381;
        for c in self.bytes() {
            result = ((result << 5).wrapping_add(result)).wrapping_add(c.into());
        }
        result
    }
}

#[derive(Default, Clone)]
struct HashCell {
    key: String, 
    value: usize,
    taken: bool,
}

struct HashTable {
    cells: Vec<HashCell>,
    taken_count: usize,
}

impl HashTable {
    fn new() -> Self {
        const INIT_CAP: usize  = 10;
        Self {
            cells: vec![HashCell::default(); INIT_CAP],
            taken_count: 0,
        }
    }

    fn extend(&mut self) {
        assert!(self.cells.len() > 0);
        let mut new_self = Self {
            cells: vec![HashCell::default(); self.cells.len()*2],
            taken_count: 0,
        };
        
        // rehash taken data from self to new_self
        for cell in self.cells.iter() {
            if cell.taken {
                let mut index = cell.key.hash() % new_self.cells.len();

                while new_self.cells[index].taken {
                    index = (index + 1) % new_self.cells.len();
                }

                new_self.cells[index].key = cell.key.clone();
                new_self.cells[index].value = cell.value;
                new_self.cells[index].taken = true;
                new_self.taken_count += 1;
            }
        }
        *self = new_self;
    }

    fn insert(&mut self, key: &String) {
        if self.taken_count >= self.cells.len() {
            self.extend();
        }

        let mut index = key.hash() % self.cells.len();

        // linear probing
        while self.cells[index].taken {
            if self.cells[index].key == *key {
                self.cells[index].value += 1;
                break;
            }
            index = (index + 1) % self.cells.len();
        }

        if !self.cells[index].taken {
            self.cells[index].key = String::from(key);
            self.cells[index].value = 1;
            self.cells[index].taken = true;
            self.taken_count += 1;
        }
    }
}

fn better_analysis(slots: &mut HashTable, words: &Vec<&str>) {
    let start_time = SystemTime::now();
    for token in words {
        slots.insert(&String::from(*token));
    }
    let end_time = SystemTime::now();
    let elapsed_time = end_time.duration_since(start_time).unwrap();

    println!("    Tokens: {}", slots.taken_count);
    slots.cells.sort_by_key(|entry| std::cmp::Reverse(entry.value));
    println!("    Top 10 tokens evaluated");
    for i in 0..10 {
        println!("      {}: ({}, {})", i, slots.cells[i].key, slots.cells[i].value);
    }
    println!("    Elapsed time: {:.3}s", elapsed_time.as_secs_f64());
}

fn main() {

    let mut file_path = String::new();
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        file_path = String::from(&args[1]);
    } else {
        println!("Usage: cargo run <file>");
        println!("ERROR: no input file provided");
    }

    let mut data_file = match read_entire_file(&file_path) {
        Ok(file) => file,
        Err(_) => return,
    };

    let mut content = String::new();

    data_file.read_to_string(&mut content).unwrap();

    println!("Analyzing ./{}", file_path);
    println!("    Size: {} bytes\n", content.len());
    //println!("{}", content);

    let words: Vec<&str> = content.trim().split_whitespace().collect();

    println!("\"Naive Analysis\"");
    naive_analysis(&mut FreqKVs::new(), &words);
    println!();

    println!("\"Better Analysis\"");
    better_analysis(&mut HashTable::new(), &words);
}
