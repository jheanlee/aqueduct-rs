/*
 * Copyright 2026 Jhe-An Lee
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import {
  Field,
  FieldDescription,
  FieldError,
  FieldGroup,
  FieldLabel,
  FieldLegend,
  FieldSet,
} from "@/components/ui/field.tsx";
import { Controller, useForm } from "react-hook-form";
import { z } from "zod";
import { settingsSchema } from "@/form-schemas/settings.ts";
import { zodResolver } from "@hookform/resolvers/zod";
import { Switch } from "@/components/ui/switch.tsx";
import { Textarea } from "@/components/ui/textarea.tsx";
import { useEffect, useState } from "react";
import { getSettings, setSettings } from "@/services/settings.ts";
import { toast } from "sonner";
import { Button } from "@/components/ui/button.tsx";
import { Separator } from "@/components/ui/separator.tsx";
import { normalizeCidr, overlapCidr } from "cidr-tools";

export const SettingsForm = () => {
  const [submitStatus, setSubmitStatus] = useState(200);

  const getSubmitStatusMessage = () => {
    switch (submitStatus) {
      case 400:
        return "Invalid request.";
      case 500:
        return "Unable to connect to the server.";
      default:
        return `An error has occurred. Error code: ${submitStatus}`;
    }
  };

  const form = useForm<z.infer<typeof settingsSchema>>({
    resolver: zodResolver(settingsSchema),
    defaultValues: {
      whitelistEnabled: false,
      blacklistEnabled: false,
      whitelist: "",
      blacklist: "",
    },
  });

  useEffect(() => {
    const getData = async () => {
      const res = await getSettings();

      if (typeof res === "number") {
        toast.error(`An error has occurred. Error code: ${res}`);
      } else {
        form.setValues({
          blacklistEnabled: res.blacklist_enabled,
          blacklist: res.blacklist.join("\n"),
          whitelistEnabled: res.whitelist_enabled,
          whitelist: res.whitelist.join("\n"),
        });
      }
    };

    void getData();
  }, [form]);

  const onSubmit = async (values: z.infer<typeof settingsSchema>) => {
    const whitelist = values.whitelist
      .split("\n")
      .filter((str) => str.length > 0);
    const blacklist = values.blacklist
      .split("\n")
      .filter((str) => str.length > 0);

    //  validation
    if (!blacklist.every((block) => normalizeCidr(block) === block)) {
      form.setError("blacklist", { message: "Invalid CIDR block." });
      return;
    }
    if (!whitelist.every((block) => normalizeCidr(block) === block)) {
      form.setError("whitelist", { message: "Invalid CIDR block." });
      return;
    }

    for (let i = 0; i < whitelist.length; i++) {
      for (let j = i + 1; j < whitelist.length; j++) {
        if (overlapCidr(whitelist[i], whitelist[j])) {
          form.setError("whitelist", {
            message: `CIDR blocs ${whitelist[i]} and ${whitelist[j]} overlap`,
          });
          return;
        }
      }
    }
    for (let i = 0; i < blacklist.length; i++) {
      for (let j = i + 1; j < blacklist.length; j++) {
        if (overlapCidr(blacklist[i], blacklist[j])) {
          form.setError("blacklist", {
            message: `CIDR blocs ${blacklist[i]} and ${blacklist[j]} overlap`,
          });
          return;
        }
      }
    }

    //  request
    const res = await setSettings({
      whitelist_enabled: values.whitelistEnabled,
      blacklist_enabled: values.blacklistEnabled,
      whitelist,
      blacklist,
    });
    setSubmitStatus(res);
    if (res === 200) {
      toast.success("Settings updated");
    }
  };

  return (
    <div>
      <form onSubmit={form.handleSubmit(onSubmit)}>
        <FieldSet>
          <FieldGroup>
            <FieldLegend>Access</FieldLegend>
            <Controller
              name="blacklistEnabled"
              control={form.control}
              render={({ field, fieldState }) => (
                <Field>
                  <FieldLabel>Blacklist</FieldLabel>
                  <div className="flex flex-row gap-3">
                    <Switch
                      checked={field.value}
                      aria-invalid={fieldState.invalid}
                      onCheckedChange={field.onChange}
                    />
                    <p className="text-sm">Enable blacklist</p>
                  </div>
                  {fieldState.invalid && (
                    <FieldError errors={[fieldState.error]} />
                  )}
                </Field>
              )}
            />
            <Controller
              name="blacklist"
              control={form.control}
              render={({ field, fieldState }) => (
                <Field>
                  <Textarea
                    placeholder={"192.168.1.10/32\n192.168.1.0/24\n10.0.0.0/8"}
                    {...field}
                  />
                  <FieldDescription>
                    Enter one CIDR block per line.
                  </FieldDescription>
                  {fieldState.invalid && (
                    <FieldError errors={[fieldState.error]} />
                  )}
                </Field>
              )}
            />
            <Controller
              name="whitelistEnabled"
              control={form.control}
              render={({ field, fieldState }) => (
                <Field>
                  <FieldLabel>Whitelist</FieldLabel>
                  <div className="flex flex-row gap-3">
                    <Switch
                      checked={field.value}
                      aria-invalid={fieldState.invalid}
                      onCheckedChange={field.onChange}
                    />
                    <p className="text-sm">Enable whitelist</p>
                  </div>
                  {fieldState.invalid && (
                    <FieldError errors={[fieldState.error]} />
                  )}
                </Field>
              )}
            />
            <Controller
              name="whitelist"
              control={form.control}
              render={({ field, fieldState }) => (
                <Field>
                  <Textarea
                    placeholder={"192.168.1.10/32\n192.168.1.0/24\n10.0.0.0/8"}
                    {...field}
                  />
                  <FieldDescription>
                    Enter one CIDR block per line. Blacklist overrides whitelist
                    if both match.
                  </FieldDescription>
                  {fieldState.invalid && (
                    <FieldError errors={[fieldState.error]} />
                  )}
                </Field>
              )}
            />
            {submitStatus !== 200 && (
              <FieldError>{getSubmitStatusMessage()}</FieldError>
            )}
          </FieldGroup>
        </FieldSet>
        <Separator className="my-3" />
        <Button type="submit">Save Changes</Button>
      </form>
    </div>
  );
};
