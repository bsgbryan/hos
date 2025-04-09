# frozen_string_literal: true

# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Copyright (c) 2025 Bryan Maynard <bsgbryan@gmail.com>

require_relative '../tools/serial/minipush'
require_relative '../tools/tests/boot_test'

require 'pty'

# Match for the last print that 'demo_payload_rpiX.img' produces
EXPECTED_PRINT = 'Echoing input now'

# Wait for request to power the target
class PowerTargetRequestTest < SubtestBase
  MINIPUSH_POWER_TARGET_REQUEST = 'Please power the target now'

  def initialize(qemu_cmd, pty_main)
    super()
    @qemu_cmd = qemu_cmd
    @pty_main = pty_main
  end

  def name
    'Waiting for request to power target'
  end

  def run(qemu_out, _qemu_in)
    expect_or_raise(qemu_out, MINIPUSH_POWER_TARGET_REQUEST)

    # Now is the time to start QEMU with the chainloader binary
    # QEMU's virtual tty connects to the MiniPush spawned on pty_main,
    # so that the two processes talk to each other
    Process.spawn(@qemu_cmd, in: @pty_main, out: @pty_main, err: '/dev/null')
  end
end

# Extend BootTest so that it listens on the output of a MiniPush instance,
# which is itself is connected to a QEMU instance instead of real hardware
class ChainbootTest < BootTest
  MINIPUSH = '../tools/serial/minipush.rb'

  def initialize(qemu_cmd, payload_path)
    super(qemu_cmd, EXPECTED_PRINT)

    @test_name = 'Boot test using MiniPush'
    @payload_path = payload_path
  end

  private

  def setup
    pty_main, pty_secondary = PTY.open

    mp_out, _mp_in = PTY.spawn("ruby #{MINIPUSH} #{pty_secondary.path} #{@payload_path}")

    # The subtests (from this class and the parents) listen on @qemu_out_wrapped
    # So point it to MiniPush's output
    @qemu_out_wrapped = PTYLoggerWrapper.new(mp_out, "\r\n")

    # Important: Run this subtest before the one in the parent class
    @console_subtests.prepend(PowerTargetRequestTest.new(@qemu_cmd, pty_main))
  end

  def finish
    super()
    @test_output.map! { |x| x.gsub(/.*\r/, '  ') }
  end
end

payload_path = ARGV.pop
qemu_cmd = ARGV.join(' ')

ChainbootTest.new(qemu_cmd, payload_path).run