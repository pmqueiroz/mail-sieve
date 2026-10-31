export interface GuideStep {
  title: string;
  body: string;
  link?: { label: string; url: string };
}

export interface ProviderGuide {
  title: string;
  intro: string;
  steps: GuideStep[];
  troubleshooting: string[];
}

export const guides: Record<string, ProviderGuide> = {
  gmail: {
    title: 'Connect a Gmail account',
    intro:
      'Gmail does not accept your normal Google password from other apps. Instead you create an app password: a separate 16-letter code that only mailsieve uses and that you can revoke at any time.',
    steps: [
      {
        title: 'Turn on 2-Step Verification',
        body: 'App passwords only exist on accounts with 2-Step Verification. If it is already on, skip this step.',
        link: { label: 'Open 2-Step Verification settings', url: 'https://myaccount.google.com/signinoptions/twosv' },
      },
      {
        title: 'Create an app password',
        body: 'Type a name such as "mailsieve" and click Create. Google shows a 16-letter code in four groups. Copy it now, Google shows it only once.',
        link: { label: 'Open App passwords', url: 'https://myaccount.google.com/apppasswords' },
      },
      {
        title: 'Make sure IMAP is enabled',
        body: 'In Gmail, open Settings, then See all settings, then Forwarding and POP/IMAP, and check that IMAP access is enabled. Most accounts have it on already.',
        link: { label: 'Open Gmail settings', url: 'https://mail.google.com/mail/u/0/#settings/fwdandpop' },
      },
      {
        title: 'Fill in the form here',
        body: 'Keep the Gmail provider selected, enter your full Gmail address, and paste the app password into the password field. Spaces are removed for you. Leave Username empty.',
      },
      {
        title: 'Connect and wait for the first sync',
        body: 'mailsieve checks the login, then indexes your mail in the background. Senders start appearing within seconds, and you can keep using the app while it works.',
      },
    ],
    troubleshooting: [
      '"Invalid credentials" almost always means the normal Google password was used. Use the 16-letter app password instead.',
      'If the app password does not work, delete it on the App passwords page and create a new one.',
      'Work or school (Google Workspace) accounts may have app passwords disabled by an administrator.',
      'Google sometimes blocks a first sign-in from a new place. Check your security alerts and approve it.',
    ],
  },
  icloud: {
    title: 'Connect an iCloud account',
    intro: 'iCloud Mail needs an app-specific password instead of your Apple ID password.',
    steps: [
      {
        title: 'Create an app-specific password',
        body: 'Sign in, open Sign-In and Security, choose App-Specific Passwords, and generate one named "mailsieve".',
        link: { label: 'Open Apple Account', url: 'https://account.apple.com' },
      },
      {
        title: 'Fill in the form here',
        body: 'Keep the iCloud provider selected, enter your iCloud email address, and paste the generated password.',
      },
    ],
    troubleshooting: ['Use the address that ends in @icloud.com, @me.com or @mac.com, not a custom domain alias.'],
  },
  yahoo: {
    title: 'Connect a Yahoo account',
    intro: 'Yahoo Mail needs an app password instead of your normal password.',
    steps: [
      {
        title: 'Generate an app password',
        body: 'Open Account security, choose Generate app password, name it "mailsieve" and copy the code.',
        link: { label: 'Open Yahoo account security', url: 'https://login.yahoo.com/account/security' },
      },
      {
        title: 'Fill in the form here',
        body: 'Keep the Yahoo provider selected, enter your Yahoo address and paste the app password.',
      },
    ],
    troubleshooting: ['If the option is missing, turn on two-step verification in the same page first.'],
  },
  fastmail: {
    title: 'Connect a Fastmail account',
    intro: 'Fastmail uses app passwords for IMAP access.',
    steps: [
      {
        title: 'Create an app password',
        body: 'Open Settings, then Privacy & Security, then App passwords. Create one with IMAP and SMTP access.',
        link: { label: 'Open Fastmail settings', url: 'https://app.fastmail.com/settings/security/apppasswords' },
      },
      {
        title: 'Fill in the form here',
        body: 'Keep the Fastmail provider selected, enter your Fastmail address and paste the app password.',
      },
    ],
    troubleshooting: ['Make sure the app password includes IMAP access.'],
  },
  outlook: {
    title: 'Connect an Outlook or Microsoft 365 account',
    intro:
      'Microsoft turned off password logins for IMAP on most accounts. The reliable way is Sign in with Microsoft, which the server administrator enables by registering an app with Microsoft.',
    steps: [
      {
        title: 'Use Sign in with Microsoft if you see the button',
        body: 'It appears at the top of this dialog when the server has Microsoft sign-in configured.',
      },
      {
        title: 'Otherwise, ask whoever runs this server',
        body: 'They need to set MAIL_SIEVE_MICROSOFT_CLIENT_ID and MAIL_SIEVE_MICROSOFT_CLIENT_SECRET. The README explains how.',
      },
    ],
    troubleshooting: ['Some work accounts still allow app passwords if the administrator permits them.'],
  },
  custom: {
    title: 'Connect another IMAP server',
    intro: 'Any provider that offers IMAP works. Look for "IMAP settings" in your provider\'s help pages.',
    steps: [
      {
        title: 'Find your IMAP server details',
        body: 'You need the server name (for example imap.example.com) and port. Port 993 with TLS is the most common.',
      },
      {
        title: 'Check whether you need an app password',
        body: 'Many providers require a separate app password for mail apps when two-step verification is on.',
      },
      {
        title: 'Fill in the form here',
        body: 'Enter the server, port, your address, and the password. Add the SMTP server too if you want unsubscribe by email to work.',
      },
    ],
    troubleshooting: ['Use security "None" only for a server on your own machine or network.'],
  },
};

export function isAuthenticationFailure(message: string): boolean {
  return /AUTHENTICATIONFAILED|invalid credentials|authentication failed|LOGIN failed/i.test(message);
}
