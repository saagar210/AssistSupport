/**
 * Template variable replacement utilities
 */

import type { CustomVariable, TemplateContext } from '../types';

// Regex to match template placeholders: {{variable_name}}
const PLACEHOLDER_REGEX = /\{\{(\w+)\}\}/g;

/**
 * Built-in template variables with their resolvers
 */
export const BUILTIN_VARIABLES: Record<string, (ctx: TemplateContext) => string> = {
  // Date/time variables
  date: () => new Date().toLocaleDateString(),
  time: () => new Date().toLocaleTimeString(),
  datetime: () => new Date().toLocaleString(),
  date_iso: () => new Date().toISOString().split('T')[0],
  timestamp: () => new Date().toISOString(),

  // Context variables (replaced with actual values or kept as placeholder)
  ticket_id: (ctx) => ctx.ticketId || '{{ticket_id}}',
  customer_name: (ctx) => ctx.customerName || '{{customer_name}}',
  agent_name: (ctx) => ctx.agentName || '{{agent_name}}',
};

/**
 * Get list of all built-in variable names
 */
export function getBuiltinVariableNames(): string[] {
  return Object.keys(BUILTIN_VARIABLES);
}

/**
 * Get description for built-in variables
 */
export const BUILTIN_VARIABLE_DESCRIPTIONS: Record<string, string> = {
  date: 'Current date in local format (e.g., 1/24/2026)',
  time: 'Current time in local format (e.g., 2:30:00 PM)',
  datetime: 'Current date and time (e.g., 1/24/2026, 2:30:00 PM)',
  date_iso: 'Current date in ISO format (e.g., 2026-01-24)',
  timestamp: 'Full ISO timestamp (e.g., 2026-01-24T14:30:00.000Z)',
  ticket_id: 'Current ticket ID (if available)',
  customer_name: 'Customer name (if available)',
  agent_name: 'Agent name (if configured)',
};

/**
 * Apply template variable replacement
 *
 * @param template - Template string containing {{variable}} placeholders
 * @param context - Context with ticket/customer/agent info
 * @param customVariables - User-defined custom variables
 * @returns Template with variables replaced
 */
export function applyTemplate(
  template: string,
  context: TemplateContext = {},
  customVariables: CustomVariable[] = []
): string {
  // Build custom variable lookup map
  const customVarMap = new Map(
    customVariables.map(v => [v.name, v.value])
  );

  return template.replace(PLACEHOLDER_REGEX, (match, varName: string) => {
    // First check built-in variables
    if (BUILTIN_VARIABLES[varName]) {
      return BUILTIN_VARIABLES[varName](context);
    }

    // Then check custom variables
    if (customVarMap.has(varName)) {
      return customVarMap.get(varName)!;
    }

    // Keep unmatched placeholders as-is
    return match;
  });
}

/**
 * Extract all variable names from a template
 *
 * @param template - Template string
 * @returns Array of unique variable names found
 */
export function extractVariables(template: string): string[] {
  const matches = template.matchAll(PLACEHOLDER_REGEX);
  const variables = new Set<string>();

  for (const match of matches) {
    variables.add(match[1]);
  }

  return Array.from(variables);
}

/**
 * Validate a custom variable name
 * Must be alphanumeric with underscores, not a reserved built-in name
 *
 * @param name - Variable name to validate
 * @returns Error message if invalid, null if valid
 */
export function validateVariableName(name: string): string | null {
  if (!name || name.trim().length === 0) {
    return 'Variable name is required';
  }

  if (!/^[a-zA-Z][a-zA-Z0-9_]*$/.test(name)) {
    return 'Variable name must start with a letter and contain only letters, numbers, and underscores';
  }

  if (BUILTIN_VARIABLES[name]) {
    return `"${name}" is a reserved built-in variable name`;
  }

  if (name.length > 50) {
    return 'Variable name must be 50 characters or less';
  }

  return null;
}

/**
 * Format a variable for insertion into template
 */
export function formatVariable(name: string): string {
  return `{{${name}}}`;
}
