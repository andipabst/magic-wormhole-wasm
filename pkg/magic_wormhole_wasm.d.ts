/* tslint:disable */
/* eslint-disable */
/**
*/
export function init(): void;
/**
* @param {HTMLInputElement} file_input
* @param {HTMLElement} output
* @returns {Promise<void>}
*/
export function send(file_input: HTMLInputElement, output: HTMLElement): Promise<void>;
/**
* @param {string} code
* @param {HTMLElement} output
* @returns {Promise<any | undefined>}
*/
export function receive(code: string, output: HTMLElement): Promise<any | undefined>;
