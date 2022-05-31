/* tslint:disable */
/* eslint-disable */
/**
* @param {WormholeConfig} config
* @param {Blob} file
* @param {string} file_name
* @param {Promise<any>} cancel
* @param {Function} progress_handler
* @returns {Promise<any>}
*/
export function send(config: WormholeConfig, file: Blob, file_name: string, cancel: Promise<any>, progress_handler: Function): Promise<any>;
/**
* @param {WormholeConfig} config
* @param {string} code
* @param {Promise<any>} cancel
* @param {Function} progress_handler
* @returns {Promise<any>}
*/
export function receive(config: WormholeConfig, code: string, cancel: Promise<any>, progress_handler: Function): Promise<any>;
/**
*/
export enum EventType {
  None,
  Progress,
  ServerWelcome,
  FileMetadata,
  ConnectedToRelay,
  Code,
}
/**
*/
export class Event {
  free(): void;
}
/**
*/
export class WormholeConfig {
  free(): void;
/**
* @param {string} rendezvous_url
* @param {string} relay_url
* @returns {WormholeConfig}
*/
  static new(rendezvous_url: string, relay_url: string): WormholeConfig;
}
