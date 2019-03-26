'use strict';

const fs = require('fs');
const dirname = require('path').dirname;
const mkdirp = require('mkdirp');

let copyCounter = 0;
const timeoutTick = 100;
const copyQueue = [];
const emptyFn = Function.prototype;

const copyFile = (source, destination) => {
  const parentDir = dirname(destination);

  return new Promise((resolve, reject) => {
    let ended = false; // Makes sure callback is called only once.
    const callback = (error) => {
      if (ended) return;
      ended = true;
      if (error) reject(error);
      else resolve();
    };

    const attempt = (error, retries) => {
      if (error != null) return callback(error);
      if (retries == null) retries = 0;

      ++copyCounter;

      let instanceError = false;
      const onError = (err) => {
        if (instanceError) return;
        instanceError = true;
        --copyCounter;
        if (retries >= 5) return callback(err);
        const code = err.code;
        if (code === 'OK' || code === 'UNKNOWN' || code === 'EMFILE') {
          copyQueue.push(() => { attempt(null, ++retries); });
        } else if (code === 'EBUSY') {
          const timeout = timeoutTick * ++retries;
          setTimeout(() => { attempt(null, retries); }, timeout);
        } else {
          return callback(err);
        }
      };

      let onClose = () => {
        if (--copyCounter < 1 && copyQueue.length) {
          const nextFn = copyQueue.shift();
          process.nextTick(nextFn);
        }
        onClose = emptyFn;
        callback();
      };

      const input = fs.createReadStream(source);
      const output = input.pipe(fs.createWriteStream(destination));
      input.on('error', onError);
      output.on('error', onError);
      output.on('close', onClose);
      output.on('finish', onClose);
    };

    fs.exists(parentDir, (exists) => {
      if (!exists) return mkdirp(parentDir, attempt);
      if (copyQueue.length) {
        copyQueue.push(attempt);
      } else {
        attempt();
      }
    });
  });
};

module.exports = copyFile;
