#!/bin/bash

echo "🚀 NeuraL Translator を起動しています..."

# Ollamaが起動していることを確認
if ! pgrep -x "ollama" > /dev/null; then
    echo "⚡ Ollamaを起動中..."
    ollama serve &
    sleep 2
fi

# 開発モードで起動
echo "🔧 開発モードで起動中..."
npm run tauri dev