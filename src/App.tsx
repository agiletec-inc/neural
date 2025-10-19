import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

interface TranslateResponse {
  translated_text: string;
}

function App() {
  const [inputText, setInputText] = useState("");
  const [translatedText, setTranslatedText] = useState("");
  const [fromLang, setFromLang] = useState("Auto");
  const [toLang, setToLang] = useState("Japanese");
  const [isTranslating, setIsTranslating] = useState(false);
  const [isHealthy, setIsHealthy] = useState(false);
  const [autoTranslate, setAutoTranslate] = useState(false);
  const [lastClipboard, setLastClipboard] = useState("");
  const [translationCache] = useState<Map<string, string>>(new Map());
  const [isCached, setIsCached] = useState(false);
  const [copiedToClipboard, setCopiedToClipboard] = useState(false);

  useEffect(() => {
    checkOllamaHealth();
    
    // Listen for global shortcut trigger
    const unlisten = listen('translate-shortcut', async () => {
      try {
        const clipboardText = await invoke<string>("get_clipboard_text");
        if (clipboardText) {
          setInputText(clipboardText);
          // Auto-translate clipboard content
          translateText(clipboardText);
        }
      } catch (error) {
        console.error("Failed to get clipboard text:", error);
      }
    });
    
    return () => {
      unlisten.then(fn => fn());
    };
  }, [fromLang, toLang]);

  // Clipboard monitoring
  useEffect(() => {
    if (!autoTranslate) return;

    const checkClipboard = async () => {
      try {
        const clipboardText = await invoke<string>("get_clipboard_text");
        if (clipboardText && clipboardText !== lastClipboard && clipboardText.trim()) {
          setLastClipboard(clipboardText);
          setInputText(clipboardText);
          translateText(clipboardText);
        }
      } catch (error) {
        console.error("Failed to check clipboard:", error);
      }
    };

    const interval = setInterval(checkClipboard, 1000); // Check every second
    return () => clearInterval(interval);
  }, [autoTranslate, lastClipboard, fromLang, toLang]);

  async function checkOllamaHealth() {
    try {
      const healthy = await invoke<boolean>("check_ollama_health");
      setIsHealthy(healthy);
    } catch (error) {
      console.error("Failed to check Ollama health:", error);
      setIsHealthy(false);
    }
  }

  async function translateText(text: string) {
    if (!text.trim()) return;
    
    // Check cache first
    const cacheKey = `${text}|${fromLang}|${toLang}`;
    const cached = translationCache.get(cacheKey);
    if (cached) {
      setTranslatedText(cached);
      setIsCached(true);
      setTimeout(() => setIsCached(false), 2000);
      return;
    }
    
    setIsTranslating(true);
    try {
      const response = await invoke<TranslateResponse>("translate", {
        text,
        fromLang: fromLang === "Auto" ? "English" : fromLang,
        toLang,
      });
      const translatedText = response.translated_text;
      setTranslatedText(translatedText);
      
      // Cache the translation
      translationCache.set(cacheKey, translatedText);
      
      // Limit cache size to 100 entries
      if (translationCache.size > 100) {
        const firstKey = translationCache.keys().next().value;
        if (firstKey !== undefined) {
          translationCache.delete(firstKey);
        }
      }
    } catch (error) {
      console.error("Translation error:", error);
      setTranslatedText("翻訳エラーが発生しました");
    } finally {
      setIsTranslating(false);
    }
  }

  async function translate() {
    await translateText(inputText);
  }

  async function copyTranslation() {
    if (translatedText) {
      try {
        await invoke("set_clipboard_text", { text: translatedText });
        setCopiedToClipboard(true);
        setTimeout(() => setCopiedToClipboard(false), 2000);
      } catch (error) {
        console.error("Failed to copy to clipboard:", error);
      }
    }
  }

  function switchLanguages() {
    if (fromLang !== "Auto") {
      const temp = fromLang;
      setFromLang(toLang);
      setToLang(temp);
      setInputText(translatedText);
      setTranslatedText("");
    }
  }

  const languages = ["Auto", "English", "Japanese", "Chinese", "Korean", "Spanish", "French", "German"];

  return (
    <div className="app">
      <header className="header">
        <div className="header-content">
          <div className="logo">
            <svg width="28" height="28" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M12 2L2 7V17L12 22L22 17V7L12 2Z" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
              <path d="M12 22V12" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
              <path d="M12 12L2 7" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
              <path d="M12 12L22 7" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
            <span>NeuraL Translator</span>
          </div>
          
          <div className="header-controls">
            <div className={`status-indicator ${isHealthy ? 'healthy' : 'unhealthy'}`}>
              <div className="status-dot"></div>
              <span>{isHealthy ? 'オンライン' : 'オフライン'}</span>
            </div>
            
            <button 
              className={`auto-translate-toggle ${autoTranslate ? 'active' : ''}`}
              onClick={() => setAutoTranslate(!autoTranslate)}
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M12 6L8 2L4 6M8 2V10M14 14H2" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
              自動翻訳
            </button>
          </div>
        </div>
      </header>

      <main className="main">
        <div className="translation-container">
          <div className="language-selector">
            <select 
              value={fromLang} 
              onChange={(e) => setFromLang(e.target.value)}
              className="language-select"
            >
              {languages.map(lang => (
                <option key={lang} value={lang}>{lang}</option>
              ))}
            </select>
            
            <button 
              className="switch-button"
              onClick={switchLanguages}
              disabled={fromLang === "Auto"}
            >
              <svg width="20" height="20" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M7 4L3 8L7 12M13 16L17 12L13 8M3 8H17M17 12H3" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
            </button>
            
            <select 
              value={toLang} 
              onChange={(e) => setToLang(e.target.value)}
              className="language-select"
            >
              {languages.filter(lang => lang !== "Auto").map(lang => (
                <option key={lang} value={lang}>{lang}</option>
              ))}
            </select>
          </div>

          <div className="translation-panels">
            <div className="panel input-panel">
              <div className="panel-header">
                <span className="panel-title">原文</span>
                <span className="char-count">{inputText.length} 文字</span>
              </div>
              <textarea
                className="text-area"
                value={inputText}
                onChange={(e) => setInputText(e.target.value)}
                placeholder="翻訳したいテキストを入力..."
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
                    translate();
                  }
                }}
              />
            </div>

            <div className="panel output-panel">
              <div className="panel-header">
                <span className="panel-title">翻訳</span>
                {isCached && <span className="cache-badge">キャッシュ</span>}
                {translatedText && (
                  <button 
                    className="copy-button"
                    onClick={copyTranslation}
                  >
                    {copiedToClipboard ? (
                      <>
                        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
                          <path d="M13.5 4.5L6 12L2.5 8.5" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                        </svg>
                        コピー済み
                      </>
                    ) : (
                      <>
                        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
                          <rect x="5" y="5" width="9" height="9" rx="1" stroke="currentColor" strokeWidth="1.5"/>
                          <path d="M11 3H3C2.44772 3 2 3.44772 2 4V12" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
                        </svg>
                        コピー
                      </>
                    )}
                  </button>
                )}
              </div>
              <textarea
                className="text-area"
                value={translatedText}
                readOnly
                placeholder="翻訳結果がここに表示されます"
              />
            </div>
          </div>

          <div className="action-bar">
            <button 
              className={`translate-button ${isTranslating ? 'loading' : ''}`}
              onClick={translate}
              disabled={isTranslating || !isHealthy || !inputText.trim()}
            >
              {isTranslating ? (
                <>
                  <span className="spinner"></span>
                  翻訳中...
                </>
              ) : (
                <>
                  <svg width="20" height="20" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <path d="M3 7H17L14 4M17 7L14 10M17 13H3L6 16M3 13L6 10" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                  </svg>
                  翻訳する
                </>
              )}
            </button>
            
            <span className="shortcut-hint">
              <kbd>⌘</kbd> + <kbd>Shift</kbd> + <kbd>T</kbd> でクイック起動
            </span>
          </div>
        </div>
      </main>
    </div>
  );
}

export default App;
