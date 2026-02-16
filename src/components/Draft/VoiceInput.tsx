import { useState, useRef, useCallback, useEffect } from 'react';
import './VoiceInput.css';

// Extend window for SpeechRecognition
interface SpeechRecognitionEvent {
  results: SpeechRecognitionResultList;
  resultIndex: number;
}

interface SpeechRecognitionResult {
  isFinal: boolean;
  [index: number]: SpeechRecognitionAlternative;
}

interface SpeechRecognitionAlternative {
  transcript: string;
  confidence: number;
}

interface SpeechRecognitionResultList {
  length: number;
  [index: number]: SpeechRecognitionResult;
}

interface SpeechRecognition extends EventTarget {
  continuous: boolean;
  interimResults: boolean;
  lang: string;
  start(): void;
  stop(): void;
  abort(): void;
  onresult: ((event: SpeechRecognitionEvent) => void) | null;
  onerror: ((event: { error: string }) => void) | null;
  onend: (() => void) | null;
}

declare global {
  interface Window {
    SpeechRecognition: new () => SpeechRecognition;
    webkitSpeechRecognition: new () => SpeechRecognition;
  }
}

interface VoiceInputProps {
  onTranscript: (text: string) => void;
  disabled?: boolean;
}

function getSpeechRecognitionClass(): (new () => SpeechRecognition) | null {
  if (typeof window === 'undefined') return null;
  return window.SpeechRecognition || window.webkitSpeechRecognition || null;
}

export function VoiceInput({ onTranscript, disabled = false }: VoiceInputProps) {
  const [listening, setListening] = useState(false);
  const [available, setAvailable] = useState(false);
  const recognitionRef = useRef<SpeechRecognition | null>(null);
  const interimRef = useRef<string>('');

  useEffect(() => {
    const SpeechRecognitionClass = getSpeechRecognitionClass();
    setAvailable(SpeechRecognitionClass !== null);
  }, []);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (recognitionRef.current) {
        recognitionRef.current.abort();
        recognitionRef.current = null;
      }
    };
  }, []);

  const startListening = useCallback(() => {
    const SpeechRecognitionClass = getSpeechRecognitionClass();
    if (!SpeechRecognitionClass) return;

    const recognition = new SpeechRecognitionClass();
    recognition.continuous = true;
    recognition.interimResults = true;
    recognition.lang = 'en-US';

    recognition.onresult = (event: SpeechRecognitionEvent) => {
      let finalTranscript = '';
      let interimTranscript = '';

      for (let i = event.resultIndex; i < event.results.length; i++) {
        const result = event.results[i];
        if (result.isFinal) {
          finalTranscript += result[0].transcript;
        } else {
          interimTranscript += result[0].transcript;
        }
      }

      if (finalTranscript) {
        onTranscript(finalTranscript);
        interimRef.current = '';
      } else {
        interimRef.current = interimTranscript;
      }
    };

    recognition.onerror = (event) => {
      console.error('Speech recognition error:', event.error);
      setListening(false);
      recognitionRef.current = null;
    };

    recognition.onend = () => {
      // Flush any remaining interim transcript
      if (interimRef.current) {
        onTranscript(interimRef.current);
        interimRef.current = '';
      }
      setListening(false);
      recognitionRef.current = null;
    };

    recognitionRef.current = recognition;
    recognition.start();
    setListening(true);
  }, [onTranscript]);

  const stopListening = useCallback(() => {
    if (recognitionRef.current) {
      recognitionRef.current.stop();
    }
  }, []);

  const handleToggle = useCallback(() => {
    if (listening) {
      stopListening();
    } else {
      startListening();
    }
  }, [listening, startListening, stopListening]);

  if (!available) {
    return null;
  }

  return (
    <button
      className={`voice-input-btn ${listening ? 'listening' : ''}`}
      onClick={handleToggle}
      disabled={disabled}
      aria-label={listening ? 'Stop voice input' : 'Start voice input'}
      title={listening ? 'Stop voice input' : 'Start voice input'}
    >
      <svg
        className="voice-input-icon"
        viewBox="0 0 24 24"
        width="16"
        height="16"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
        <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
        <line x1="12" y1="19" x2="12" y2="23" />
        <line x1="8" y1="23" x2="16" y2="23" />
      </svg>
      {listening && <span className="voice-input-pulse" />}
    </button>
  );
}
