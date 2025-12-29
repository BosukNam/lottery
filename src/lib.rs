use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotteryDrawing {
    pub round: u32,
    pub numbers: [u8; 6],
    pub bonus: u8,
}

#[wasm_bindgen]
pub struct LotteryEngine {
    drawings: Vec<LotteryDrawing>,
}

#[wasm_bindgen]
impl LotteryEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(json_data: &str) -> Result<LotteryEngine, JsValue> {
        let drawings: Vec<LotteryDrawing> = serde_json::from_str(json_data)
            .map_err(|e| JsValue::from_str(&format!("JSON 파싱 오류: {}", e)))?;

        Ok(LotteryEngine { drawings })
    }

    /// 5개 세트의 로또 번호 생성
    #[wasm_bindgen(js_name = generateNumbersSets)]
    pub fn generate_numbers_sets(&self) -> JsValue {
        let sets = self.generate_sets(5);
        serde_wasm_bindgen::to_value(&sets).unwrap()
    }

    /// 특정 번호를 포함한 5개 세트 생성
    #[wasm_bindgen(js_name = generateNumbersSetsWithRequired)]
    pub fn generate_numbers_sets_with_required(&self, required: Vec<u8>) -> Result<JsValue, JsValue> {
        if required.is_empty() || required.len() > 6 {
            return Err(JsValue::from_str("필수 번호는 1-6개 사이여야 합니다."));
        }

        for &num in &required {
            if num < 1 || num > 45 {
                return Err(JsValue::from_str("번호는 1-45 사이여야 합니다."));
            }
        }

        let mut unique_check = HashSet::new();
        for &num in &required {
            if !unique_check.insert(num) {
                return Err(JsValue::from_str("중복된 번호가 있습니다."));
            }
        }

        let sets = self.generate_sets_with_required(&required, 5)
            .map_err(|e| JsValue::from_str(&e))?;

        Ok(serde_wasm_bindgen::to_value(&sets).unwrap())
    }

    /// 빈도 기반 번호 추천 (낮은 빈도순)
    #[wasm_bindgen(js_name = getNumberFrequency)]
    pub fn get_number_frequency(&self) -> JsValue {
        let frequency = self.calculate_frequency();
        serde_wasm_bindgen::to_value(&frequency).unwrap()
    }

    /// 새 회차 추가
    #[wasm_bindgen(js_name = addNewDrawing)]
    pub fn add_new_drawing(&mut self, round: u32, numbers: Vec<u8>, bonus: u8) -> Result<(), JsValue> {
        if numbers.len() != 6 {
            return Err(JsValue::from_str("6개의 번호를 입력해주세요."));
        }

        let numbers_array: [u8; 6] = numbers.try_into()
            .map_err(|_| JsValue::from_str("번호 변환 오류"))?;

        let drawing = LotteryDrawing {
            round,
            numbers: numbers_array,
            bonus,
        };

        self.drawings.push(drawing);
        self.drawings.sort_by_key(|d| d.round);
        Ok(())
    }

    /// 현재 저장된 회차 범위 조회
    #[wasm_bindgen(js_name = getRoundRange)]
    pub fn get_round_range(&self) -> JsValue {
        if self.drawings.is_empty() {
            return JsValue::NULL;
        }

        let min_round = self.drawings.iter().map(|d| d.round).min().unwrap();
        let max_round = self.drawings.iter().map(|d| d.round).max().unwrap();

        serde_wasm_bindgen::to_value(&(min_round, max_round, self.drawings.len())).unwrap()
    }

    /// 현재 데이터를 JSON으로 내보내기
    #[wasm_bindgen(js_name = exportToJson)]
    pub fn export_to_json(&self) -> String {
        serde_json::to_string(&self.drawings).unwrap()
    }
}

// Internal methods
impl LotteryEngine {
    fn get_used_combinations(&self) -> HashSet<[u8; 6]> {
        let mut used_combinations = HashSet::new();

        // 1등 번호 조합
        for drawing in &self.drawings {
            let mut sorted_numbers = drawing.numbers.clone();
            sorted_numbers.sort();
            used_combinations.insert(sorted_numbers);
        }

        // 2등 번호 조합 (1등 5개 + 보너스)
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

    fn generate_sets(&self, count: usize) -> Vec<[u8; 6]> {
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

    fn generate_sets_with_required(&self, required_numbers: &[u8], count: usize) -> Result<Vec<[u8; 6]>, String> {
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

    fn calculate_frequency(&self) -> Vec<(u8, usize)> {
        let mut frequency = [0usize; 46];

        // 1등 번호 빈도
        for drawing in &self.drawings {
            for &num in &drawing.numbers {
                if num >= 1 && num <= 45 {
                    frequency[num as usize] += 1;
                }
            }
        }

        // 보너스 번호 빈도 (2등 영향)
        for drawing in &self.drawings {
            if drawing.bonus >= 1 && drawing.bonus <= 45 {
                frequency[drawing.bonus as usize] += 1;
            }
        }

        let mut freq_pairs: Vec<(u8, usize)> = (1..=45)
            .map(|num| (num, frequency[num as usize]))
            .collect();

        freq_pairs.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
        freq_pairs
    }
}

// WASM 의존성 추가
#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
}
