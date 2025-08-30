use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{File, read_dir};
use std::io::{Read, Write};
use std::path::Path;
use rand::seq::SliceRandom;
use rand::thread_rng;
use encoding_rs::EUC_KR;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LotteryDrawing {
    round: u32,
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

    fn parse_excel_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("엑셀 파일 파싱 중: {}", file_path);
        
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
        writeln!(file, "회차,당첨번호1,당첨번호2,당첨번호3,당첨번호4,당첨번호5,당첨번호6,보너스번호")?;
        
        // 데이터 작성 (회차 순으로 정렬)
        let mut sorted_drawings = self.drawings.clone();
        sorted_drawings.sort_by_key(|d| d.round);
        
        for drawing in sorted_drawings {
            writeln!(
                file,
                "{},{},{},{},{},{},{},{}",
                drawing.round,
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

    fn load_from_text_file(&mut self, file_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
        if !Path::new(file_path).exists() {
            println!("텍스트 파일이 존재하지 않습니다: {}", file_path);
            return Ok(false);
        }
        
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        self.drawings.clear();
        
        for (line_num, line) in contents.lines().enumerate() {
            if line_num == 0 { continue; } // 헤더 스킵
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 8 {
                if let (Ok(round), Ok(n1), Ok(n2), Ok(n3), Ok(n4), Ok(n5), Ok(n6), Ok(bonus)) = (
                    parts[0].parse::<u32>(),
                    parts[1].parse::<u8>(),
                    parts[2].parse::<u8>(),
                    parts[3].parse::<u8>(),
                    parts[4].parse::<u8>(),
                    parts[5].parse::<u8>(),
                    parts[6].parse::<u8>(),
                    parts[7].parse::<u8>(),
                ) {
                    let drawing = LotteryDrawing {
                        round,
                        numbers: [n1, n2, n3, n4, n5, n6],
                        bonus,
                    };
                    self.drawings.push(drawing);
                }
            }
        }
        
        println!("텍스트 파일에서 {}개 회차 로드됨", self.drawings.len());
        Ok(true)
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

    fn add_new_drawing(&mut self, round: u32, numbers: [u8; 6], bonus: u8) {
        let drawing = LotteryDrawing {
            round,
            numbers,
            bonus,
        };
        self.drawings.push(drawing);
        self.drawings.sort_by_key(|d| d.round);
    }

    fn parse_all_excel_files(&mut self, static_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        let entries = read_dir(static_dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(extension) = path.extension() {
                if extension == "xls" || extension == "xlsx" {
                    if let Some(path_str) = path.to_str() {
                        self.parse_excel_file(path_str)?;
                    }
                }
            }
        }
        
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = LotteryParser::new();
    
    // 기존 데이터가 있는지 확인하고 로드
    let data_exists = parser.load_from_text_file("lottery_data.txt")?;
    
    // lottery_data.txt가 없거나 비어있을 때만 엑셀 파일들을 파싱
    if !data_exists || parser.drawings.is_empty() {
        println!("기존 데이터가 없어 엑셀 파일을 파싱합니다.");
        parser.parse_all_excel_files("static")?;
        
        // 파싱한 데이터가 있으면 텍스트 파일로 저장
        if !parser.drawings.is_empty() {
            parser.save_to_text_file("lottery_data.txt")?;
        }
    } else {
        println!("기존 데이터를 사용합니다.");
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
                print!("회차: ");
                std::io::stdout().flush()?;
                let mut round_input = String::new();
                std::io::stdin().read_line(&mut round_input)?;
                let round = round_input.trim().parse::<u32>()?;
                
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
                
                parser.add_new_drawing(round, numbers, bonus);
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
