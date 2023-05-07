import {
  workspace,
  EventEmitter,
  ExtensionContext,
  window,
  Range,
  TextDocumentChangeEvent,
  commands,
  Selection,
} from "vscode";

import {
  Disposable,
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient;

export async function activate(context: ExtensionContext) {
  let disposable = commands.registerCommand("helloworld.helloWorld", async uri => {
    let document = await workspace.openTextDocument(uri);
    await window.showTextDocument(document);

    window.activeTextEditor.document
    let editor = window.activeTextEditor;
    let range = new Range(1, 1, 1, 1)
    editor.selection = new Selection(range.start, range.end);
  });

  context.subscriptions.push(disposable);

  const traceOutputChannel = window.createOutputChannel("Memonoa Language Server trace");
  const command = process.env.SERVER_PATH;
  console.log(command);
  const run: Executable = {
    command,
    options: {
      env: {
        ...process.env,
        RUST_LOG: "debug",
      },
    },
  };
  const serverOptions: ServerOptions = {
    run,
    debug: run,
  };
  // If the extension is launched in debug mode then the debug server options are used
  // Otherwise the run options are used
  // Options to control the language client
  let clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "markdown" }],
    traceOutputChannel,
  };

  // Create the language client and start the client.
  client = new LanguageClient("memonoa-language-server", "memonoa language server", serverOptions, clientOptions);
  // activateInlayHints(context);
  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}

export function activateInlayHints(ctx: ExtensionContext) {
  const maybeUpdater = {
    hintsProvider: null as Disposable | null,
    updateHintsEventEmitter: new EventEmitter<void>(),

    async onConfigChange() {
      this.dispose();

      const event = this.updateHintsEventEmitter.event;
      // this.hintsProvider = languages.registerInlayHintsProvider(
      //   { scheme: "file", language: "nrs" },
      //   // new (class implements InlayHintsProvider {
      //   //   onDidChangeInlayHints = event;
      //   //   resolveInlayHint(hint: InlayHint, token: CancellationToken): ProviderResult<InlayHint> {
      //   //     const ret = {
      //   //       label: hint.label,
      //   //       ...hint,
      //   //     };
      //   //     return ret;
      //   //   }
      //   //   async provideInlayHints(
      //   //     document: TextDocument,
      //   //     range: Range,
      //   //     token: CancellationToken
      //   //   ): Promise<InlayHint[]> {
      //   //     const hints = (await client
      //   //       .sendRequest("custom/inlay_hint", { path: document.uri.toString() })
      //   //       .catch(err => null)) as [number, number, string][];
      //   //     if (hints == null) {
      //   //       return [];
      //   //     } else {
      //   //       return hints.map(item => {
      //   //         const [start, end, label] = item;
      //   //         let startPosition = document.positionAt(start);
      //   //         let endPosition = document.positionAt(end);
      //   //         return {
      //   //           position: endPosition,
      //   //           paddingLeft: true,
      //   //           label: [
      //   //             {
      //   //               value: `${label}`,
      //   //               // location: {
      //   //               //   uri: document.uri,
      //   //               //   range: new Range(1, 0, 1, 0)
      //   //               // }
      //   //               command: {
      //   //                 title: "hello world",
      //   //                 command: "helloworld.helloWorld",
      //   //                 arguments: [document.uri],
      //   //               },
      //   //             },
      //   //           ],
      //   //         };
      //   //       });
      //   //     }
      //   //   }
      //   // })()
      // );
    },

    onDidChangeTextDocument({ contentChanges, document }: TextDocumentChangeEvent) {
      // debugger
      // this.updateHintsEventEmitter.fire();
    },

    dispose() {
      this.hintsProvider?.dispose();
      this.hintsProvider = null;
      this.updateHintsEventEmitter.dispose();
    },
  };

  workspace.onDidChangeConfiguration(maybeUpdater.onConfigChange, maybeUpdater, ctx.subscriptions);
  workspace.onDidChangeTextDocument(maybeUpdater.onDidChangeTextDocument, maybeUpdater, ctx.subscriptions);

  maybeUpdater.onConfigChange().catch(console.error);
}
