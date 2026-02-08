import { forwardRef, useCallback, useEffect, useRef } from "react";

interface TextAreaProps
  extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  error?: boolean;
}

export const TextArea = forwardRef<HTMLTextAreaElement, TextAreaProps>(
  function TextArea({ error, className = "", onChange, ...props }, ref) {
    const internalRef = useRef<HTMLTextAreaElement | null>(null);

    const setRefs = useCallback(
      (el: HTMLTextAreaElement | null) => {
        internalRef.current = el;
        if (typeof ref === "function") {
          ref(el);
        } else if (ref) {
          ref.current = el;
        }
      },
      [ref],
    );

    const adjustHeight = useCallback(() => {
      const textarea = internalRef.current;
      if (textarea) {
        textarea.style.height = "auto";
        textarea.style.height = `${textarea.scrollHeight}px`;
      }
    }, []);

    useEffect(() => {
      adjustHeight();
    }, [adjustHeight, props.value]);

    const handleChange = useCallback(
      (e: React.ChangeEvent<HTMLTextAreaElement>) => {
        adjustHeight();
        onChange?.(e);
      },
      [adjustHeight, onChange],
    );

    return (
      <textarea
        ref={setRefs}
        className={`min-h-[80px] w-full resize-none rounded-lg border bg-background px-3 py-2 text-sm text-foreground outline-none transition-colors placeholder:text-muted-foreground focus:ring-1 focus:ring-accent ${
          error
            ? "border-destructive focus:ring-destructive"
            : "border-border"
        } ${className}`}
        onChange={handleChange}
        {...props}
      />
    );
  },
);
