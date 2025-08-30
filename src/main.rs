use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use rand::seq::SliceRandom;
use rand::thread_rng;
use encoding_rs::EUC_KR;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LotteryDrawing {
    round: u32,
    date: String,
    numbers: [u8; 6],  // 당첨번호 6개
    bonus: u8,         // 보너스번호
}

struct LotteryParser {
    drawings: Vec<LotteryDrawing>,
}

impl LotteryParser {
    fn new() -> Self {
        Self {
            drawings: Vec::new(),
        }
    }

    fn parse_html_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("HTML 파일 파싱 중: {}", file_path);
        
        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        // EUC-KR 인코딩을 UTF-8로 변환
        let (contents, _, _) = EUC_KR.decode(&buffer);
        
        let document = Html::parse_document(&contents);
        let row_selector = Selector::parse("tr").unwrap();
        
        for row in document.select(&row_selector) {
            let cells: Vec<String> = row
                .select(&Selector::parse("td").unwrap())
                .map(|cell| cell.inner_html().trim().to_string())
                .collect();
            
            // 데이터가 있는 행인지 확인 - 더 유연한 조건
            if cells.len() >= 17 {
                // 첫 번째 또는 두 번째 셀에서 회차 찾기
                let mut round_opt: Option<u32> = None;
                let mut round_index = 0;
                
                for i in 0..=1 {
                    if i < cells.len() {
                        if let Ok(round) = cells[i].parse::<u32>() {
                            if round > 0 && round <= 2000 {  // 합리적인 회차 범위
                                round_opt = Some(round);
                                round_index = i;
                                break;
                            }
                        }
                    }
                }
                
                if let Some(round) = round_opt {
                    // 날짜는 회차 다음 셀
                    let date_index = round_index + 1;
                    if date_index >= cells.len() {
                        continue;
                    }
                    let date = cells[date_index].clone();
                    
                    // 당첨번호를 찾기 - 맨 뒤에서부터 7개 셀에서 찾기
                    let start_index = if cells.len() >= 7 { cells.len() - 7 } else { 0 };
                    
                    let mut numbers = [0u8; 6];
                    let mut bonus = 0u8;
                    let mut valid_count = 0;
                    
                    // 뒤에서부터 7개 셀에서 숫자 찾기
                    for i in start_index..cells.len() {
                        if let Ok(num) = cells[i].parse::<u8>() {
                            if num >= 1 && num <= 45 {
                                if valid_count < 6 {
                                    numbers[valid_count] = num;
                                } else if valid_count == 6 {
                                    bonus = num;
                                }
                                valid_count += 1;
                            }
                        }
                    }
                    
                    // 정확히 7개(당첨번호 6개 + 보너스 1개)가 파싱되었는지 확인
                    if valid_count == 7 && bonus > 0 {
                        let drawing = LotteryDrawing {
                            round,
                            date,
                            numbers,
                            bonus,
                        };
                        self.drawings.push(drawing);
                        println!("파싱 완료: {}회차 - 번호: {:?}, 보너스: {}", round, numbers, bonus);
                    }
                }
            }
        }
        
        Ok(())
    }

    fn save_to_text_file(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(file_path)?;
        
        // 헤더 작성
        writeln!(file, "회차,날짜,당첨번호1,당첨번호2,당첨번호3,당첨번호4,당첨번호5,당첨번호6,보너스번호")?;
        
        // 데이터 작성 (회차 순으로 정렬)
        let mut sorted_drawings = self.drawings.clone();
        sorted_drawings.sort_by_key(|d| d.round);
        
        for drawing in sorted_drawings {
            writeln!(
                file,
                "{},{},{},{},{},{},{},{},{}",
                drawing.round,
                drawing.date,
                drawing.numbers[0],
                drawing.numbers[1],
                drawing.numbers[2],
                drawing.numbers[3],
                drawing.numbers[4],
                drawing.numbers[5],
                drawing.bonus
            )?;
        }
        
        println!("텍스트 파일 저장 완료: {}", file_path);
        println!("총 {}개의 회차 데이터 저장됨", self.drawings.len());
        Ok(())
    }

    fn load_from_text_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !Path::new(file_path).exists() {
            return Ok(());
        }
        
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        self.drawings.clear();
        
        for (line_num, line) in contents.lines().enumerate() {
            if line_num == 0 { continue; } // 헤더 스킵
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 9 {
                if let (Ok(round), Ok(n1), Ok(n2), Ok(n3), Ok(n4), Ok(n5), Ok(n6), Ok(bonus)) = (
                    parts[0].parse::<u32>(),
                    parts[2].parse::<u8>(),
                    parts[3].parse::<u8>(),
                    parts[4].parse::<u8>(),
                    parts[5].parse::<u8>(),
                    parts[6].parse::<u8>(),
                    parts[7].parse::<u8>(),
                    parts[8].parse::<u8>(),
                ) {
                    let drawing = LotteryDrawing {
                        round,
                        date: parts[1].to_string(),
                        numbers: [n1, n2, n3, n4, n5, n6],
                        bonus,
                    };
                    self.drawings.push(drawing);
                }
            }
        }
        
        println!("텍스트 파일에서 {}개 회차 로드됨", self.drawings.len());
        Ok(())
    }

    fn generate_new_numbers(&self) -> [u8; 6] {
        let mut used_combinations = HashSet::new();
        
        // 기존 1등 번호 조합들을 HashSet에 저장
        for drawing in &self.drawings {
            let mut sorted_numbers = drawing.numbers.clone();
            sorted_numbers.sort();
            used_combinations.insert(sorted_numbers);
        }
        
        // 2등 번호 조합도 제외 (1등 5개 + 보너스 1개)
        for drawing in &self.drawings {
            for i in 0..6 {
                let mut second_place_combo = drawing.numbers.clone();
                second_place_combo[i] = drawing.bonus;
                second_place_combo.sort();
                used_combinations.insert(second_place_combo);
            }
        }
        
        let mut rng = thread_rng();
        let all_numbers: Vec<u8> = (1..=45).collect();
        
        loop {
            let mut selected: Vec<u8> = all_numbers.choose_multiple(&mut rng, 6).cloned().collect();
            selected.sort();
            
            let selected_array: [u8; 6] = selected.try_into().unwrap();
            
            if !used_combinations.contains(&selected_array) {
                return selected_array;
            }
        }
    }

    fn add_new_drawing(&mut self, round: u32, date: String, numbers: [u8; 6], bonus: u8) {
        let drawing = LotteryDrawing {
            round,
            date,
            numbers,
            bonus,
        };
        self.drawings.push(drawing);
        self.drawings.sort_by_key(|d| d.round);
    }
}

fn read_line_with_escape() -> Result<Option<String>, Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut input = String::new();
    
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. }) = event::read()? {
                match code {
                    KeyCode::Esc => {
                        disable_raw_mode()?;
                        println!("\n메인 메뉴로 돌아갑니다.");
                        return Ok(None);
                    }
                    KeyCode::Enter => {
                        disable_raw_mode()?;
                        println!();
                        return Ok(Some(input));
                    }
                    KeyCode::Backspace => {
                        if !input.is_empty() {
                            input.pop();
                            print!("\r회차 (ESC: 메인메뉴): {}", input);
                            std::io::stdout().flush()?;
                        }
                    }
                    KeyCode::Char(c) => {
                        input.push(c);
                        print!("\r회차 (ESC: 메인메뉴): {}", input);
                        std::io::stdout().flush()?;
                    }
                    _ => {}
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = LotteryParser::new();
    
    // lottery_data.txt 파일이 이미 존재하는지 확인
    if Path::new("lottery_data.txt").exists() {
        // 기존 데이터 로드
        parser.load_from_text_file("lottery_data.txt")?;
        println!("기존 데이터 파일을 로드했습니다 ({} 회차)", parser.drawings.len());
    } else {
        println!("데이터 파일이 없습니다. HTML 파일들을 파싱합니다...");
        
        // HTML 파일들 파싱
        if Path::new("static/1-600.xls").exists() {
            parser.parse_html_file("static/1-600.xls")?;
        }
        
        if Path::new("static/601-1186.xls").exists() {
            parser.parse_html_file("static/601-1186.xls")?;
        }
        
        // 텍스트 파일로 저장
        parser.save_to_text_file("lottery_data.txt")?;
    }
    
    println!("\n=== 로또 번호 추첨기 ===");
    println!("1. 새로운 로또 번호 추첨");
    println!("2. 신규 회차 추가");
    println!("3. 종료");
    
    loop {
        print!("\n선택하세요 (1-3): ");
        std::io::stdout().flush()?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "1" => {
                let new_numbers = parser.generate_new_numbers();
                println!("\n추천 로또 번호: {:?}", new_numbers);
                println!("(기존 1등, 2등 당첨번호 제외)");
            }
            "2" => {
                print!("회차 (ESC: 메인메뉴): ");
                std::io::stdout().flush()?;
                
                let round = match read_line_with_escape()? {
                    Some(input) => match input.trim().parse::<u32>() {
                        Ok(r) => r,
                        Err(_) => {
                            println!("올바른 숫자를 입력해주세요.");
                            continue;
                        }
                    },
                    None => continue,
                };
                
                print!("날짜 (예: 2024.01.01): ");
                std::io::stdout().flush()?;
                let mut date_input = String::new();
                std::io::stdin().read_line(&mut date_input)?;
                let date = date_input.trim().to_string();
                
                print!("1등 번호 6개 (공백으로 구분): ");
                std::io::stdout().flush()?;
                let mut numbers_input = String::new();
                std::io::stdin().read_line(&mut numbers_input)?;
                let number_parts: Vec<u8> = numbers_input
                    .trim()
                    .split_whitespace()
                    .map(|s| s.parse().unwrap())
                    .collect();
                
                if number_parts.len() != 6 {
                    println!("6개의 번호를 입력해주세요.");
                    continue;
                }
                
                let numbers: [u8; 6] = number_parts.try_into().unwrap();
                
                print!("보너스 번호: ");
                std::io::stdout().flush()?;
                let mut bonus_input = String::new();
                std::io::stdin().read_line(&mut bonus_input)?;
                let bonus = bonus_input.trim().parse::<u8>()?;
                
                parser.add_new_drawing(round, date, numbers, bonus);
                parser.save_to_text_file("lottery_data.txt")?;
                
                println!("{}회차 데이터가 추가되었습니다.", round);
            }
            "3" => {
                println!("프로그램을 종료합니다.");
                break;
            }
            _ => {
                println!("올바른 번호를 입력해주세요 (1-3).");
            }
        }
    }
    
    Ok(())
}