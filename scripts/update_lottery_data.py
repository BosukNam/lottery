#!/usr/bin/env python3
"""
로또 당첨번호를 자동으로 업데이트하는 스크립트

네이버 검색 결과에서 최신 당첨번호를 크롤링
(동행복권 API가 GitHub Actions에서 차단되어 대안으로 사용)
"""

import json
import re
import time
from pathlib import Path

# 프로젝트 루트 기준 lottery_data.json 파일 경로들
DATA_FILES = [
    "docs/lottery_data.json",
    "lottery_data.json",
    "www/lottery_data.json",
]

# 네이버 검색 URL
NAVER_SEARCH_URL = "https://search.naver.com/search.naver?query=로또+당첨번호"
# 동행복권 API (폴백용)
DHLOTTERY_API_URL = "https://www.dhlottery.co.kr/common.do?method=getLottoNumber&drwNo={}"


def get_project_root():
    """프로젝트 루트 디렉토리 반환"""
    script_dir = Path(__file__).parent
    return script_dir.parent


def load_lottery_data(filepath):
    """lottery_data.json 파일 로드"""
    with open(filepath, "r", encoding="utf-8") as f:
        return json.load(f)


def save_lottery_data(filepath, data):
    """lottery_data.json 파일 저장"""
    with open(filepath, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)


def fetch_from_naver():
    """네이버 검색 결과에서 최신 로또 당첨번호 크롤링"""
    try:
        from playwright.sync_api import sync_playwright

        print("[INFO] 네이버에서 최신 로또 당첨번호 조회 중...")

        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True)
            context = browser.new_context(
                user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
                locale="ko-KR"
            )
            page = context.new_page()

            print(f"[Playwright] {NAVER_SEARCH_URL} 접속 중...")
            page.goto(NAVER_SEARCH_URL, wait_until="domcontentloaded", timeout=30000)

            # 페이지 로딩 대기
            time.sleep(2)

            # 페이지 내용 가져오기
            content = page.content()

            # 회차 번호 추출 (예: "1207회")
            round_match = re.search(r'(\d{4})회', content)
            if not round_match:
                print("[ERROR] 회차 번호를 찾을 수 없습니다.")
                browser.close()
                return None

            round_no = int(round_match.group(1))
            print(f"[INFO] 발견된 회차: {round_no}회")

            # 당첨번호 추출 - 네이버 로또 위젯에서 번호 찾기
            # 다양한 패턴 시도
            numbers = []

            # 방법 1: ball 클래스에서 번호 추출
            ball_matches = re.findall(r'class="[^"]*ball[^"]*"[^>]*>(\d{1,2})<', content)
            if ball_matches and len(ball_matches) >= 7:
                numbers = [int(n) for n in ball_matches[:7]]

            # 방법 2: 연속된 숫자 패턴 찾기 (1-45 범위)
            if not numbers or len(numbers) < 7:
                num_pattern = re.findall(r'>(\d{1,2})<', content)
                valid_nums = [int(n) for n in num_pattern if 1 <= int(n) <= 45]
                # 연속으로 나타나는 7개 숫자 그룹 찾기
                for i in range(len(valid_nums) - 6):
                    candidate = valid_nums[i:i+7]
                    if len(set(candidate)) == 7:  # 모두 다른 숫자
                        numbers = candidate
                        break

            browser.close()

            if numbers and len(numbers) >= 7:
                main_numbers = sorted(numbers[:6])
                bonus = numbers[6]
                print(f"[INFO] 당첨번호: {main_numbers}, 보너스: {bonus}")
                return {
                    "round": round_no,
                    "numbers": main_numbers,
                    "bonus": bonus
                }
            else:
                print(f"[WARN] 당첨번호를 파싱할 수 없습니다. 발견된 숫자: {numbers}")
                return None

    except ImportError:
        print("[ERROR] Playwright가 설치되지 않았습니다.")
        return None
    except Exception as e:
        print(f"[ERROR] 네이버 크롤링 오류: {e}")
        return None


def fetch_from_dhlottery(round_no):
    """동행복권 API에서 당첨번호 조회 (폴백)"""
    try:
        import requests

        url = DHLOTTERY_API_URL.format(round_no)
        headers = {
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        }

        response = requests.get(url, headers=headers, timeout=10)

        if response.status_code == 200:
            try:
                data = response.json()
                if data.get("returnValue") == "success":
                    return {
                        "round": data["drwNo"],
                        "numbers": sorted([
                            data["drwtNo1"], data["drwtNo2"], data["drwtNo3"],
                            data["drwtNo4"], data["drwtNo5"], data["drwtNo6"]
                        ]),
                        "bonus": data["bnusNo"]
                    }
            except:
                pass
        return None
    except Exception as e:
        print(f"[WARN] 동행복권 API 오류: {e}")
        return None


def get_latest_round(data):
    """현재 데이터에서 최신 회차 번호 반환"""
    if not data:
        return 0
    return max(item["round"] for item in data)


def update_lottery_data():
    """로또 데이터 업데이트 메인 함수"""
    project_root = get_project_root()

    # 메인 데이터 파일 경로 (docs/lottery_data.json)
    main_data_file = project_root / DATA_FILES[0]

    if not main_data_file.exists():
        print(f"데이터 파일을 찾을 수 없습니다: {main_data_file}")
        return False

    # 현재 데이터 로드
    data = load_lottery_data(main_data_file)
    latest_round = get_latest_round(data)
    print(f"현재 최신 회차: {latest_round}")

    # 네이버에서 최신 당첨번호 가져오기
    naver_result = fetch_from_naver()

    if naver_result is None:
        print("[WARN] 네이버 크롤링 실패. 동행복권 API 시도...")
        # 동행복권 API로 폴백
        next_round = latest_round + 1
        naver_result = fetch_from_dhlottery(next_round)

    if naver_result is None:
        print("\n새로운 회차를 가져올 수 없습니다.")
        return False

    # 이미 있는 회차인지 확인
    if naver_result["round"] <= latest_round:
        print(f"\n회차 {naver_result['round']}은 이미 저장되어 있습니다.")
        print("새로운 회차가 없습니다. 데이터가 최신 상태입니다.")
        return False

    # 누락된 회차가 있는지 확인하고 순차적으로 추가
    new_rounds_added = 0
    for round_no in range(latest_round + 1, naver_result["round"] + 1):
        if round_no == naver_result["round"]:
            result = naver_result
        else:
            # 중간 회차는 동행복권 API로 시도
            result = fetch_from_dhlottery(round_no)

        if result:
            data.append(result)
            new_rounds_added += 1
            print(f"회차 {result['round']} 추가됨: {result['numbers']} + 보너스 {result['bonus']}")
            time.sleep(0.5)

    if new_rounds_added == 0:
        print("\n새로운 회차가 없습니다. 데이터가 최신 상태입니다.")
        return False

    # 회차 순으로 정렬
    data.sort(key=lambda x: x["round"])

    # 모든 데이터 파일에 저장
    for data_file in DATA_FILES:
        filepath = project_root / data_file
        if filepath.exists() or data_file == DATA_FILES[0]:
            save_lottery_data(filepath, data)
            print(f"저장됨: {filepath}")

    print(f"\n총 {new_rounds_added}개의 새 회차가 추가되었습니다.")
    print(f"현재 최신 회차: {get_latest_round(data)}")
    return True


if __name__ == "__main__":
    update_lottery_data()
    exit(0)
