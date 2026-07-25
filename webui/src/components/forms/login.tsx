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

import { z } from "zod";
import { Controller, useForm } from "react-hook-form";
import { Input } from "@/components/ui/input.tsx";
import { login } from "@/services/auth.ts";
import { useNavigate } from "react-router";
import { paths } from "@/config/paths.ts";
import {
  Field,
  FieldError,
  FieldGroup,
  FieldLabel,
  FieldSet,
} from "@/components/ui/field.tsx";
import { useState } from "react";
import { Button } from "@/components/ui/button.tsx";
import { zodResolver } from "@hookform/resolvers/zod";
import { loginSchema } from "@/form-schemas/login.ts";

export const LoginForm = () => {
  const navigate = useNavigate();
  const [submitStatus, setSubmitStatus] = useState<number>(200);
  const getSubmitStatusMessage = () => {
    switch (submitStatus) {
      case 401:
        return "Incorrect username or password.";
      case 500:
        return "Unable to connect to the server.";
      default:
        return `An error has occurred. Error code: ${submitStatus}`;
    }
  };

  const form = useForm<z.infer<typeof loginSchema>>({
    resolver: zodResolver(loginSchema),
    defaultValues: {
      username: "",
      password: "",
    },
  });

  const onSubmit = async (values: z.infer<typeof loginSchema>) => {
    const res = await login(values);
    setSubmitStatus(res);
    if (res === 200) {
      navigate(paths.root.dashboard.getHref());
    }
  };

  return (
    <div className="w-full h-full flex justify-center items-center">
      <form
        onSubmit={form.handleSubmit(onSubmit)}
        className="w-70 md:w-100 h-70 flex flex-col gap-4 content-center"
      >
        <FieldSet data-invalid={submitStatus !== 200}>
          <FieldGroup>
            <Controller
              name="username"
              control={form.control}
              render={({ field, fieldState }) => (
                <Field>
                  <FieldLabel>Username</FieldLabel>
                  <Input
                    type="text"
                    placeholder="user"
                    aria-invalid={fieldState.invalid}
                    {...field}
                  />
                  {fieldState.invalid && (
                    <FieldError errors={[fieldState.error]} />
                  )}
                </Field>
              )}
            />
            <Controller
              name="password"
              control={form.control}
              render={({ field, fieldState }) => (
                <Field>
                  <FieldLabel>Password</FieldLabel>
                  <Input
                    type="password"
                    placeholder="••••••••"
                    aria-invalid={fieldState.invalid}
                    {...field}
                  />
                  {fieldState.invalid && (
                    <FieldError errors={[fieldState.error]} />
                  )}
                </Field>
              )}
            />
            {submitStatus !== 200 && (
              <FieldError>{getSubmitStatusMessage()}</FieldError>
            )}
            <Field>
              <Button type="submit">Submit</Button>
            </Field>
          </FieldGroup>
        </FieldSet>
      </form>
    </div>
  );
};
