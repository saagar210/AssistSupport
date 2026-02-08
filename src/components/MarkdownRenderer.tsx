import { useState, useCallback } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeHighlight from "rehype-highlight";
import { Copy, Check } from "lucide-react";

interface MarkdownRendererProps {
  content: string;
}

function CopyButton({ code }: { code: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    await navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }, [code]);

  return (
    <button
      onClick={handleCopy}
      className="absolute right-2 top-2 rounded p-1 text-muted-foreground opacity-0 transition-opacity hover:bg-muted hover:text-foreground group-hover/code:opacity-100"
      title="Copy code"
      data-testid="copy-code-button"
    >
      {copied ? <Check size={14} /> : <Copy size={14} />}
    </button>
  );
}

export function MarkdownRenderer({ content }: MarkdownRendererProps) {
  return (
    <div className="prose-vm" data-testid="markdown-renderer">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeHighlight]}
        components={{
          pre({ children, ...props }) {
            // Extract code text from children for copy button
            const codeText = extractText(children);
            return (
              <div className="group/code relative">
                <pre {...props} className="overflow-x-auto rounded-md bg-muted p-3 text-sm">
                  {children}
                </pre>
                <CopyButton code={codeText} />
              </div>
            );
          },
          code({ children, className, ...props }) {
            const isInline = !className;
            if (isInline) {
              return (
                <code
                  className="rounded bg-muted px-1.5 py-0.5 text-sm font-mono"
                  {...props}
                >
                  {children}
                </code>
              );
            }
            return (
              <code className={className} {...props}>
                {children}
              </code>
            );
          },
          a({ href, children, ...props }) {
            return (
              <a
                href={href}
                target="_blank"
                rel="noopener noreferrer"
                className="text-accent underline underline-offset-2 hover:text-accent/80"
                {...props}
              >
                {children}
              </a>
            );
          },
          table({ children, ...props }) {
            return (
              <div className="overflow-x-auto">
                <table
                  className="w-full border-collapse text-sm"
                  {...props}
                >
                  {children}
                </table>
              </div>
            );
          },
          th({ children, ...props }) {
            return (
              <th
                className="border border-border bg-muted px-3 py-1.5 text-left text-xs font-semibold"
                {...props}
              >
                {children}
              </th>
            );
          },
          td({ children, ...props }) {
            return (
              <td
                className="border border-border px-3 py-1.5 text-xs"
                {...props}
              >
                {children}
              </td>
            );
          },
          blockquote({ children, ...props }) {
            return (
              <blockquote
                className="border-l-2 border-accent/50 pl-4 italic text-muted-foreground"
                {...props}
              >
                {children}
              </blockquote>
            );
          },
          ul({ children, ...props }) {
            return (
              <ul className="list-disc pl-6 space-y-1" {...props}>
                {children}
              </ul>
            );
          },
          ol({ children, ...props }) {
            return (
              <ol className="list-decimal pl-6 space-y-1" {...props}>
                {children}
              </ol>
            );
          },
          h1({ children, ...props }) {
            return <h1 className="text-xl font-bold mt-4 mb-2" {...props}>{children}</h1>;
          },
          h2({ children, ...props }) {
            return <h2 className="text-lg font-bold mt-3 mb-2" {...props}>{children}</h2>;
          },
          h3({ children, ...props }) {
            return <h3 className="text-base font-semibold mt-2 mb-1" {...props}>{children}</h3>;
          },
          p({ children, ...props }) {
            return <p className="leading-relaxed mb-2 last:mb-0" {...props}>{children}</p>;
          },
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
}

function extractText(node: React.ReactNode): string {
  if (typeof node === "string") return node;
  if (typeof node === "number") return String(node);
  if (!node) return "";
  if (Array.isArray(node)) return node.map(extractText).join("");
  if (typeof node === "object" && node !== null && "props" in node) {
    const element = node as { props: { children?: React.ReactNode } };
    return extractText(element.props.children);
  }
  return "";
}
