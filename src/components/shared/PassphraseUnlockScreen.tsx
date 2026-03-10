import { FormEvent, useState } from 'react';
import { Button } from './Button';
import { Icon } from './Icon';
import './PassphraseUnlockScreen.css';

interface PassphraseUnlockScreenProps {
  error: string | null;
  onUnlock: (passphrase: string) => Promise<void>;
}

export function PassphraseUnlockScreen({
  error,
  onUnlock,
}: PassphraseUnlockScreenProps) {
  const [passphrase, setPassphrase] = useState('');
  const [submitting, setSubmitting] = useState(false);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!passphrase.trim()) {
      return;
    }

    setSubmitting(true);
    try {
      await onUnlock(passphrase);
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <main className="passphrase-screen">
      <section
        className="passphrase-card"
        aria-labelledby="passphrase-unlock-title"
        aria-describedby="passphrase-unlock-description"
      >
        <div className="passphrase-icon" aria-hidden="true">
          <Icon name="shield" size={28} />
        </div>
        <h1 id="passphrase-unlock-title">Unlock AssistSupport</h1>
        <p id="passphrase-unlock-description" className="passphrase-description">
          This workspace uses passphrase-protected key storage. Enter the passphrase to unlock your
          local database and encrypted tokens.
        </p>

        <form className="passphrase-form" onSubmit={handleSubmit}>
          <label className="passphrase-label" htmlFor="passphrase-input">
            Passphrase
          </label>
          <input
            id="passphrase-input"
            className="passphrase-input"
            type="password"
            autoComplete="current-password"
            value={passphrase}
            onChange={event => setPassphrase(event.target.value)}
            placeholder="Enter your passphrase"
            disabled={submitting}
            required
          />

          {error && (
            <div className="passphrase-error" role="alert">
              <Icon name="alertCircle" size={16} />
              <span>{error}</span>
            </div>
          )}

          <Button
            type="submit"
            variant="primary"
            size="large"
            loading={submitting}
            disabled={!passphrase.trim()}
          >
            Unlock
          </Button>
        </form>
      </section>
    </main>
  );
}
