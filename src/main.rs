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

                for i in 0..=1 {
                    if i < cells.len() {
                        if let Ok(round) = cells[i].parse::<u32>() {
                            if round > 0 && round <= 2000 {  // 합리적인 회차 범위
                                round_opt = Some(round);
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

    fn get_used_combinations(&self) -> HashSet<[u8; 6]> {
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
        
        used_combinations
    }



    fn generate_numbers_sets_with_required(&self, required_numbers: &[u8], count: usize) -> Result<Vec<[u8; 6]>, String> {
        if required_numbers.is_empty() || required_numbers.len() > 6 {
            return Err("필수 번호는 1-6개 사이여야 합니다.".to_string());
        }

        for &num in required_numbers {
            if num < 1 || num > 45 {
                return Err("번호는 1-45 사이여야 합니다.".to_string());
            }
        }

        // 중복 체크
        let mut unique_check = HashSet::new();
        for &num in required_numbers {
            if !unique_check.insert(num) {
                return Err("중복된 번호가 있습니다.".to_string());
            }
        }

        let used_combinations = self.get_used_combinations();
        let mut rng = thread_rng();
        let remaining_numbers: Vec<u8> = (1..=45)
            .filter(|&n| !required_numbers.contains(&n))
            .collect();

        let needed_count = 6 - required_numbers.len();
        let mut results = Vec::new();
        let mut attempts = 0;
        let max_attempts = count * 1000;

        while results.len() < count && attempts < max_attempts {
            attempts += 1;
            let mut selected = required_numbers.to_vec();
            let additional: Vec<u8> = remaining_numbers
                .choose_multiple(&mut rng, needed_count)
                .cloned()
                .collect();
            selected.extend(additional);
            selected.sort();

            let selected_array: [u8; 6] = selected.try_into().unwrap();

            if !used_combinations.contains(&selected_array) && !results.contains(&selected_array) {
                results.push(selected_array);
            }
        }

        if results.len() < count {
            return Err(format!("조건에 맞는 번호 조합을 {}개 찾을 수 없습니다. ({}개만 생성됨)", count, results.len()));
        }

        Ok(results)
    }

    fn generate_numbers_sets(&self, count: usize) -> Vec<[u8; 6]> {
        let used_combinations = self.get_used_combinations();
        let mut rng = thread_rng();
        let all_numbers: Vec<u8> = (1..=45).collect();
        let mut results = Vec::new();
        let mut attempts = 0;
        let max_attempts = count * 1000;
        
        while results.len() < count && attempts < max_attempts {
            attempts += 1;
            let mut selected: Vec<u8> = all_numbers.choose_multiple(&mut rng, 6).cloned().collect();
            selected.sort();
            
            let selected_array: [u8; 6] = selected.try_into().unwrap();
            
            if !used_combinations.contains(&selected_array) && !results.contains(&selected_array) {
                results.push(selected_array);
            }
        }
        
        results
    }

    fn get_number_frequency(&self) -> Vec<(u8, usize)> {
        let mut frequency = [0usize; 46]; // 인덱스 0은 사용하지 않고 1-45만 사용
        
        // 1등 번호 빈도 계산
        for drawing in &self.drawings {
            for &num in &drawing.numbers {
                if num >= 1 && num <= 45 {
                    frequency[num as usize] += 1;
                }
            }
        }
        
        // 보너스 번호도 2등에 영향을 주므로 포함
        for drawing in &self.drawings {
            if drawing.bonus >= 1 && drawing.bonus <= 45 {
                frequency[drawing.bonus as usize] += 1;
            }
        }
        
        // (번호, 빈도) 쌍으로 변환하고 빈도 기준 오름차순 정렬
        let mut freq_pairs: Vec<(u8, usize)> = (1..=45)
            .map(|num| (num, frequency[num as usize]))
            .collect();
        
        freq_pairs.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
        freq_pairs
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

    fn get_round_range(&self) -> Option<(u32, u32)> {
        if self.drawings.is_empty() {
            return None;
        }

        let min_round = self.drawings.iter().map(|d| d.round).min().unwrap();
        let max_round = self.drawings.iter().map(|d| d.round).max().unwrap();
        Some((min_round, max_round))
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

fn get_number_input(prompt: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    print!("{}", prompt);
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let numbers: Result<Vec<u8>, _> = input
        .trim()
        .split_whitespace()
        .map(|s| s.parse::<u8>())
        .collect();

    Ok(numbers?)
}

fn show_menu() {
    println!("\n=== 로또 번호 추첨기 ===");
    println!("1. 새로운 로또 번호 추첨 (5개 세트)");
    println!("2. 특정 수 포함 번호 추첨 (반자동, 5개 세트)");
    println!("3. 수 추천 (빈도 기반)");
    println!("4. 신규 회차 추가");
    println!("5. 종료");
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
    
    show_menu();
    
    loop {
        print!("\n선택하세요 (1-5): ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(0) => {
                println!("\n프로그램을 종료합니다.");
                break;
            }
            Ok(_) => {},
            Err(_) => {
                println!("\n입력 오류가 발생했습니다. 프로그램을 종료합니다.");
                break;
            }
        }

        match input.trim() {
            "1" => {
                let number_sets = parser.generate_numbers_sets(5);
                println!("\n=== 추천 로또 번호 5개 세트 ===");
                for (i, numbers) in number_sets.iter().enumerate() {
                    println!("{}: {:?}", i + 1, numbers);
                }
                println!("(기존 1등, 2등 당첨번호 제외)");
                show_menu();
            }
            "2" => {
                match get_number_input("포함할 번호들 (공백으로 구분): ") {
                    Ok(required_numbers) => {
                        match parser.generate_numbers_sets_with_required(&required_numbers, 5) {
                            Ok(number_sets) => {
                                println!("\n=== 특정 수 포함 추천 로또 번호 5개 세트 ===");
                                for (i, numbers) in number_sets.iter().enumerate() {
                                    println!("{}: {:?}", i + 1, numbers);
                                }
                                println!("포함된 수: {:?}", required_numbers);
                                println!("(기존 1등, 2등 당첨번호 제외)");
                            }
                            Err(error) => {
                                println!("오류: {}", error);
                            }
                        }
                    }
                    Err(_) => {
                        println!("올바른 번호를 입력해주세요.");
                    }
                }
                show_menu();
            }
            "3" => {
                let frequency = parser.get_number_frequency();
                println!("\n=== 빈도 기반 수 추천 ===");
                println!("가장 낮은 빈도순으로 정렬:");

                let mut current_index = 0;
                loop {
                    if current_index >= frequency.len() {
                        println!("모든 수를 추천했습니다.");
                        break;
                    }

                    let (number, freq) = frequency[current_index];
                    println!("추천 수: {} (빈도: {}회)", number, freq);
                    current_index += 1;

                    print!("다음 수를 보시겠습니까? (Enter: 다음, q: 종료): ");
                    std::io::stdout().flush()?;
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;

                    if input.trim().to_lowercase() == "q" {
                        break;
                    }
                }
                show_menu();
            }
            "4" => {
                println!("\n=== 신규 회차 추가 ===");

                // 현재 저장된 회차 범위 표시
                match parser.get_round_range() {
                    Some((min_round, max_round)) => {
                        println!("현재 저장된 회차: {}회 ~ {}회 (총 {}개)",
                                min_round, max_round, parser.drawings.len());
                    }
                    None => {
                        println!("현재 저장된 데이터가 없습니다.");
                    }
                }

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

                // 업데이트된 회차 범위 표시
                if let Some((min_round, max_round)) = parser.get_round_range() {
                    println!("업데이트된 회차: {}회 ~ {}회 (총 {}개)",
                            min_round, max_round, parser.drawings.len());
                }

                show_menu();
            }
            "5" => {
                println!("프로그램을 종료합니다.");
                break;
            }
            _ => {
                println!("올바른 번호를 입력해주세요 (1-5).");
                show_menu();
            }
        }
    }
    
    Ok(())
}
