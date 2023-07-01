import type { AppInfoWithState } from './types'
import { AppState } from './types'

export default [
  {
    app_id: '12',
    name: 'Poll',
    description: 'Poll app where you can create crazy cool polls. This is a very long description for the pepe.',
    submitter_uri: 'Jonas Arndt',
    source_code_url: 'https://example.com?t=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
    image: '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',
    version: 1,
    cached: false,
    state: AppState.Initial,
    rights_obtained: false,
  },
  {
    app_id: '13',
    name: '2048',
    description: 'The popular 2048 game comes to dc!',
    submitter_uri: 'SomeDude',
    source_code_url: 'https://mycompany.com/the/code',
    image: 'iVBORw0KGgoAAAANSUhEUgAAAIAAAACABAMAAAAxEHz4AAAAHlBMVEXtwwDwzjHv1VHy22704In145r26bL38dX39eX69fMbIa03AAAACXBIWXMAAAsTAAALEwEAmpwYAAACQElEQVRo3u1Vv1fbQAy2nV+wpS0PyOa2C9kCLGULr0PLRqc+Npa+V2909JZseANKcr7/Fukk2WefnTJTfUNiWbrP0nfSXRQpFAqFQqFQKBQKxX+B9+dnU8/4NO2Mij/7Yb7jt7W2/MHGTzDMovaOrX12D4McPZcdBJlFlEtnXDjD1F+6YoIkdx67CNZPyEFhAzYexZuI54Q9f3sSAOBXZ/xcivdYCHLxtGWIwWN+oQy3FGa+fgPjmt05E0Bq5Wn0AcxlO4OhLVOXx4oSXjjjgZwjKQ60fKKE7oIajtbws+fWQNiGZOFSLzyCO/pbhdswpTVAsE/yQb5bqg8qKhzBZBcBYt8RzDhDURGWrYlgRNTHJFUHaOmc1ctpT1CMlAhi1xyxOELcOHmvuFEyihuiFkSAAZvpd2nLsJ2pD4Ag5Wj8n2PCTDDGHujsRCl2Wy9koriwJhKC6Ivroz8985iRRjdcIpWCEtYEh45g2b0e+yUNCDL3smhMiUn7JHyO2iUMqJ2YAF6Z87xHxJHk1hBxRntKBIkLGfaomAlxYxtzlFAIJjQLJ52dOK54vUZyO3sPgN27X2E6t9QZT2EP5NVbbuUYWzm2NR5wqlLyhCIc1UfYnjdMfQSb9vqkqAfEH+eAYEFaBhmAy3x8h2geKE2CGTXhJNQgqcO6jjTZBSjO0JG22kkwbx+qQjCoopY7CYb8uG4TVIf3NtpJwDPnXSxCINTX/yAIrrZqGg8K7wZ89U37qstVoVAoFAqFQqFQvDG8AKSlmPH5RxokAAAAAElFTkSuQmCC',
    version: 1,
    cached: false,
    state: AppState.Downloading,
  },
  {
    app_id: '14',
    name: 'Calendar',
    description: 'A simple calendar app for Delta Chat.',
    submitter_uri: 'TheDev',
    source_code_url: 'https://foo.org/',
    image: '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',
    version: 1,
    cached: false,
    state: AppState.DownloadCancelled,
  },
  {
    app_id: '15',
    name: 'Chess Board',
    description: 'A webxdc chess game for two players and many observers :)',
    submitter_uri: 'root',
    source_code_url: 'https://corp.com/',
    image: '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',
    version: 1,
    cached: false,
    state: AppState.Received,
  },
  {
    app_id: '16',
    name: 'Draw',
    description: 'A webxdc tool that allows to share draws.',
    submitter_uri: 'ArtCreations',
    source_code_url: 'https://artc.com/',
    image: 'iVBORw0KGgoAAAANSUhEUgAAAH0AAAB9BAMAAAB9rnEWAAAAHlBMVEVifYx7k56Joaugr7ivvcTAztTW297l6uz1+fz9//zvID+YAAAACXBIWXMAAAsTAAALEwEAmpwYAAACf0lEQVRYw+1YPU/cQBBdc+ZMCfkgd52jCJF0JEWEO1KRdBEoUuggDVyHKCK540II2Q7wGXv/bez9HHOBmx13aF8BNvLDOx/vza4ZCwgICAgIeBqI1nrRP3JR7dDp26JBvUWlrwuJkkhf4ooviBG803Rx2+/1oiLx3wqLVUrlueNTKhA7uvhKWf8Xxz9k1AWcjdqfp4y4gJINye9ngzZxK3Q+2y4Yy3o04CBlLG/5KVmBA6lAuoJHfQTYYNLyb/otn5p+s/we6Zt0w39xzMXVvocFyNdPze2uUsNvND/pqPezkdOZlwTv9M2m02PqYyHTeT9AGuIyXP4E8CuP6t3BVPgZEmy+vMOf4qu385/X25w+iiHQno6+/nmifmMcfdw+WYBMivoTYy/RCThoHzyXl5niH9lrjKNyG34EGzdBShpYj8penbq/F8j0lSB7v3RXIi1pxXZq3M05xxVwbPOUdXpGzvX6b4OLR/mZqdMSjD7adXO5WCze1JqYij6GfVwsLH+b/ih3Moy58Oa/ks/OZOhdFeH4ufPwA+HBl/HrNlCOMRQ+fJn/LR39dN4DFvFl/S+/u+Ld84BF/OT+kzD6+luDPa8NmLm/5EgH5V27GqvrH9j+N6ZhVJDr2YPVnytX5UZ5qeUwwxhoDrePiUlEjJ5ACTw/ZOZyiN+SyOJXqfOgU+Mr57gB2vyDP3r+2cGb+eyon72GblrarHjv6G3WVB28d0LLJuoRev7N86/N8m9p/MI0lf+BQMZfv3/OiQeiSPjvXx5QI21DDI5TpPMcsKMZgQ6Os8TjjLWTi37H+WqVxtefE2r694yNZgVX5K8ZbRO9+RA+SQUEBAQEPDH8AwiiSrCeN9eaAAAAAElFTkSuQmCC',
    version: 1,
    cached: false,
    state: AppState.Initial,
  },
] as AppInfoWithState[]
