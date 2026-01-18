#!/usr/bin/env python3
"""
동행복권 API를 통해 로또 당첨번호를 자동으로 업데이트하는 스크립트

API 엔드포인트: https://www.dhlottery.co.kr/common.do?method=getLottoNumber&drwNo={회차}
"""

import json
import time
from pathlib import Path

# cloudscraper를 사용하여 CloudFlare 봇 차단 우회
try:
    import cloudscraper
    scraper = cloudscraper.create_scraper(
        browser={
            'browser': 'chrome',
            'platform': 'windows',
            'desktop': True
        }
    )
    print("[INFO] cloudscraper 사용")
except ImportError:
    import requests
    scraper = requests.Session()
    print("[WARN] cloudscraper 없음, requests 사용")

API_URL = "https://www.dhlottery.co.kr/common.do?method=getLottoNumber&drwNo={}"

# 프로젝트 루트 기준 lottery_data.json 파일 경로들
DATA_FILES = [
    "docs/lottery_data.json",
    "lottery_data.json",
    "www/lottery_data.json",
]


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


def fetch_lottery_result(round_no, max_retries=3):
    """동행복권 API에서 특정 회차 당첨번호 조회 (재시도 로직 포함)"""
    url = API_URL.format(round_no)

    for attempt in range(max_retries):
        try:
            response = scraper.get(url, timeout=15)

            # 응답 상태 코드 확인
            print(f"[DEBUG] 회차 {round_no} - 상태 코드: {response.status_code}")

            # 403, 503 등 서버 오류 시 재시도
            if response.status_code in (403, 429, 500, 502, 503, 504):
                raise Exception(f"서버 오류: HTTP {response.status_code}")

            # 응답이 비어있는지 확인
            response_text = response.text.strip()
            if not response_text:
                raise ValueError("빈 응답을 받았습니다")

            # Content-Type 확인 및 디버깅
            content_type = response.headers.get("Content-Type", "")
            print(f"[DEBUG] 회차 {round_no} - Content-Type: {content_type}")

            # JSON이 아닌 응답 감지 (HTML 오류 페이지 등)
            if "text/html" in content_type.lower():
                print(f"[DEBUG] HTML 응답 감지 - 첫 200자: {response_text[:200]}")
                raise ValueError("API가 HTML을 반환했습니다 (봇 차단 또는 서버 오류)")

            # JSON 형식인지 기본 검사
            if not response_text.startswith("{"):
                print(f"[DEBUG] 비정상 응답 - 첫 200자: {response_text[:200]}")
                raise ValueError(f"JSON이 아닌 응답: {response_text[:100]}")

            data = response.json()

            if data.get("returnValue") != "success":
                # 추첨되지 않은 회차는 정상적인 경우
                print(f"[DEBUG] 회차 {round_no} - returnValue: {data.get('returnValue')}")
                return None

            # API 응답을 프로젝트 형식으로 변환
            return {
                "round": data["drwNo"],
                "numbers": sorted([
                    data["drwtNo1"],
                    data["drwtNo2"],
                    data["drwtNo3"],
                    data["drwtNo4"],
                    data["drwtNo5"],
                    data["drwtNo6"],
                ]),
                "bonus": data["bnusNo"],
            }
        except Exception as e:
            if attempt < max_retries - 1:
                wait_time = 2 ** (attempt + 1)  # 2, 4초 대기
                print(f"회차 {round_no} 조회 실패 (시도 {attempt + 1}/{max_retries}): {e}")
                print(f"{wait_time}초 후 재시도...")
                time.sleep(wait_time)
            else:
                print(f"회차 {round_no} 조회 실패: {e}")
                return None

    return None


def verify_api_connection(test_round):
    """API 연결 상태를 검증하기 위해 기존 회차 테스트"""
    print(f"\n[INFO] API 연결 검증 중 (회차 {test_round} 테스트)...")

    result = fetch_lottery_result(test_round)

    if result is not None:
        print(f"[INFO] API 연결 검증 성공: 회차 {result['round']} - {result['numbers']}")
        return True
    else:
        print("[ERROR] API 연결 검증 실패: 기존 회차도 조회할 수 없습니다.")
        print("[ERROR] 동행복권 사이트에서 봇 차단 중일 수 있습니다.")
        return False


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

    # API 연결 검증 (기존 회차 테스트)
    if not verify_api_connection(latest_round):
        print("\n[WARN] API 검증 실패. 그래도 새 회차 조회를 시도합니다...")

    # 새로운 회차 확인 및 추가
    new_rounds_added = 0
    next_round = latest_round + 1

    # 요청 간 대기
    time.sleep(1)

    while True:
        print(f"\n회차 {next_round} 조회 중...")
        result = fetch_lottery_result(next_round)

        if result is None:
            print(f"회차 {next_round}은 아직 추첨되지 않았거나 조회할 수 없습니다.")
            break

        data.append(result)
        new_rounds_added += 1
        print(f"회차 {next_round} 추가됨: {result['numbers']} + 보너스 {result['bonus']}")
        next_round += 1

        # 연속 요청 시 짧은 대기
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
    # 새로운 회차가 없어도 정상 종료 (데이터가 최신 상태이므로 에러가 아님)
    exit(0)
