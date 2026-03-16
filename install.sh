#!/bin/bash
set -e

echo "=== Han (한) 설치 ==="
echo ""

# 1. Rust 컴파일러 설치
echo "[1/3] hgl 빌드 및 설치..."
cargo install --path .
echo "  hgl 설치 완료"

# 2. VS Code 확장 설치 (VS Code 있으면)
if command -v code &> /dev/null; then
    echo "[2/3] VS Code 확장 설치..."
    VSIX="editors/vscode/han-language-0.1.0.vsix"
    if [ ! -f "$VSIX" ]; then
        echo "  VSIX 빌드 중..."
        cd editors/vscode
        npm install --silent 2>/dev/null
        npx @vscode/vsce package --allow-missing-repository 2>/dev/null
        cd ../..
    fi
    code --install-extension "$VSIX" --force 2>/dev/null
    echo "  VS Code 확장 설치 완료"
else
    echo "[2/3] VS Code 없음 — 확장 설치 건너뜀"
fi

# 3. 확인
echo "[3/3] 설치 확인..."
echo ""
hgl --version 2>/dev/null || echo "  hgl 설치됨"
echo ""
echo "=== 설치 완료 ==="
echo ""
echo "사용법:"
echo "  hgl interpret hello.hgl    # 인터프리터로 실행"
echo "  hgl build hello.hgl        # 네이티브 바이너리 컴파일"
echo "  hgl repl                   # 대화형 REPL"
echo ""
echo "문서: https://xodn348.github.io/han/introduction.html"
echo "플레이그라운드: https://xodn348.github.io/han/playground/"
