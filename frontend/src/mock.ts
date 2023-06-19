import type { AppInfo } from './bindings/AppInfo'
import { AppState } from './types'

export default [
    {
        id: 12,
        name: 'Poll',
        description: 'Poll app where you can create crazy cool polls. This is a very long description for the pepe.',
        author_name: 'Jonas Arndt',
        author_email: 'xxde@you.de',
        source_code_url: 'https://example.com',
        image: 'iVBORw0KGgoAAAANSUhEUgAAAJYAAACWCAIAAACzY+a1AAAEB0lEQVR4nO3YQU/yShiG4SmlBSwYjEIQCyaSqmHl//8NLNgZSaORAmJQxCC0dihzFs3hEPQkX8KXlid5rl1r9YW5w2RQ63Q6gpBl0n4BtC8mhMeE8JgQHhPCY0J4TAiPCeExITwmhMeE8JgQHhPCY0J4TAiPCeExITwmhMeE8JgQHhPCY0J4TAiPCeExITwmhMeE8JgQHhPCY0J4TAiPCeExITwmhMeE8JgQHhPCY0J4TAiPCeExITwmhMeE8JgQHhPCY0J4TAiPCeExITwmhMeE8JgQHhPCY0J4TAiPCeExITwmhMeE8JgQHhPCY0J4TAiPCeExITwmhMeE8JgQHhPCY0J42bRfwH88zxNCNBqN+HK5XPZ6ve0HHMexLEsIoZQaj8fT6TSKouPjY9u2s9m93kiKo/d3KAnH4/H7+/vp6enmThAEpmm22+1fH57NZq1WS9f1fr//9PTkOA7i6L8i/Y00DEPXdSeTiWma2/eDIMjn8z+fV0pNJpNarZbP5w3DaDabi8VisVhgjf6L0k+4XC5N07y9vc3lctv3/28dfd9fr9fxtiaEMAzDNM2ddZzNZt1udz6fx5ePj48PDw9KqQRGJy/9jbRcLpfL5Z/34/W6v7+XUhYKhXq9Hq+dlFIIYRjG5knDMMIw3PmbJycng8Hg5uZmNpvN5/Pr62tN0xIYnbz0P4W/iqJISmmapuM47Xb76OjIdd0gCIQQ6/VaCLHdQ9O0n58w27bX67XnecPhsFarFQqFxEYn7EAT6rp+d3fXbDaz2Ww2m724uMjlcm9vb+LfFdxeOKVUJrP7RnRdt2374+Mjl8tVq9UkRyfsQBP+ZJpmvI/FR4/VarX5kZRye3Pb8H1fCBGGYRRFCY9O0oEm/Pr66na739/f8aVSanPEyOfzmUxmc4iQUoZhuDlibPi+//r6Wq/XdV2Pv/YlNjphB5rQsqxCoeB5XhiGq9VqMBhEUVSpVIQQmUzm7OxsNBr5vi+l7Pf7lmXtrKNS6vn5uVgsVqvVRqPx+fk5nU6TGZ289E+kv9I07erqajQa9Xq9+BzvOM7m/yDn5+dKKdd1hRClUuny8nLn119eXqSUrVZLCFEsFiuVynA4LJVKf7Lp7Tk6eVqn00n7NdBeDnQjpT/HhPCYEB4TwmNCeEwIjwnhMSE8JoTHhPCYEB4TwmNCeEwIjwnhMSE8JoTHhPCYEB4TwmNCeEwIjwnhMSE8JoTHhPCYEB4TwmNCeEwIjwnhMSE8JoTHhPCYEB4TwmNCeEwIjwnhMSE8JoTHhPCYEB4TwmNCeEwIjwnhMSE8JoTHhPCYEB4TwmNCeEwIjwnhMSE8JoTHhPCYEB4TwmNCeEwIjwnhMSE8JoTHhPCYEN4/cnznZiQb6hsAAAAASUVORK5CYII=',
        version: '1.11',
        state: AppState.Initial,
    },
    {
        id: 13,
        name: '2048',
        description: 'The popular 2048 game comes to dc!',
        author_name: 'SomeDude',
        author_email: 'somedude123@you.de',
        source_code_url: 'https://mycompany.com/the/code',
        image: 'iVBORw0KGgoAAAANSUhEUgAAAIAAAACABAMAAAAxEHz4AAAAHlBMVEXtwwDwzjHv1VHy22704In145r26bL38dX39eX69fMbIa03AAAACXBIWXMAAAsTAAALEwEAmpwYAAACQElEQVRo3u1Vv1fbQAy2nV+wpS0PyOa2C9kCLGULr0PLRqc+Npa+V2909JZseANKcr7/Fukk2WefnTJTfUNiWbrP0nfSXRQpFAqFQqFQKBQKxX+B9+dnU8/4NO2Mij/7Yb7jt7W2/MHGTzDMovaOrX12D4McPZcdBJlFlEtnXDjD1F+6YoIkdx67CNZPyEFhAzYexZuI54Q9f3sSAOBXZ/xcivdYCHLxtGWIwWN+oQy3FGa+fgPjmt05E0Bq5Wn0AcxlO4OhLVOXx4oSXjjjgZwjKQ60fKKE7oIajtbws+fWQNiGZOFSLzyCO/pbhdswpTVAsE/yQb5bqg8qKhzBZBcBYt8RzDhDURGWrYlgRNTHJFUHaOmc1ctpT1CMlAhi1xyxOELcOHmvuFEyihuiFkSAAZvpd2nLsJ2pD4Ag5Wj8n2PCTDDGHujsRCl2Wy9koriwJhKC6Ivroz8985iRRjdcIpWCEtYEh45g2b0e+yUNCDL3smhMiUn7JHyO2iUMqJ2YAF6Z87xHxJHk1hBxRntKBIkLGfaomAlxYxtzlFAIJjQLJ52dOK54vUZyO3sPgN27X2E6t9QZT2EP5NVbbuUYWzm2NR5wqlLyhCIc1UfYnjdMfQSb9vqkqAfEH+eAYEFaBhmAy3x8h2geKE2CGTXhJNQgqcO6jjTZBSjO0JG22kkwbx+qQjCoopY7CYb8uG4TVIf3NtpJwDPnXSxCINTX/yAIrrZqGg8K7wZ89U37qstVoVAoFAqFQqFQvDG8AKSlmPH5RxokAAAAAElFTkSuQmCC',
        version: '0.2.0',
        state: AppState.Initial,
    },
] as AppInfo
